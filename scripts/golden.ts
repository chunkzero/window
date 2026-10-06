// Builds each plugin/test/golden case with the TypeScript Window plugin and verifies the pack output
// and Kotlin bindings against the committed expected.sha256. Usage: node scripts/golden.ts [--update] [case...]
// `--update` rewrites expected.sha256 instead.
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { cpSync, existsSync, mkdirSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

interface Case {
    packFormat: number;
    hudShaders?: boolean;
    anvilFieldSprite?: string;
    kotlinPackage?: string;
    /** Required with `kotlinPackage`. */
    kotlinTarget?: "agnostic" | "minestom" | "multistom";
}

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const pluginDir = join(root, "plugin");
const goldenDir = join(pluginDir, "test", "golden");
const buildDir = join(root, "build", "golden");
const rpp = process.env["RPP"] ?? "rpp";
const kotlinDir = "kotlin-out";

function write(path: string, contents: string): void {
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, contents);
}

function copyTree(from: string, to: string): void {
    if (existsSync(from)) {
        cpSync(from, to, { recursive: true });
    }
}

function build(dir: string): void {
    const result = spawnSync(rpp, ["-C", dir, "build", "--no-squash"], { encoding: "utf8" });
    if (result.status !== 0) {
        throw new Error(`rpp build failed in ${relative(root, dir)}:\n${result.stdout}${result.stderr}`);
    }
}

function tsProject(caseDir: string, dir: string, options: Case): void {
    copyTree(join(caseDir, "shared"), join(dir, "src"));
    copyTree(join(caseDir, "ts"), join(dir, "src"));
    write(join(dir, "rpp.json"), JSON.stringify({ dependencies: { window: `path:${pluginDir}` } }));
    const windowOptions = [
        `hudShaders: ${options.hudShaders === true}`,
        ...(options.anvilFieldSprite === undefined
            ? []
            : [`anvilFieldSprite: ${JSON.stringify(options.anvilFieldSprite)}`]),
        ...(options.kotlinPackage === undefined
            ? []
            : [
                  `kotlin: { package: ${JSON.stringify(options.kotlinPackage)}, output: ${JSON.stringify(kotlinDir)}, target: ${JSON.stringify(options.kotlinTarget)} }`,
              ]),
    ];
    write(
        join(dir, "rpp.config.ts"),
        `import { defineConfig } from "#rpp/config";
import window from "#plugins/window";

export default defineConfig({
    pack: { name: "golden", format: ${options.packFormat} },
    build: { workers: 1, squash: { enabled: false } },
    plugins: [window({ ${windowOptions.join(", ")} })],
});
`,
    );
}

/** Every file under `<dir>/<sub>`, keyed by its path relative to `dir`. */
function readTree(dir: string, sub: string): Map<string, Buffer> {
    const files = new Map<string, Buffer>();
    const top = join(dir, sub);
    if (!existsSync(top)) {
        return files;
    }
    for (const entry of readdirSync(top, { recursive: true, withFileTypes: true })) {
        if (entry.isFile()) {
            const path = join(entry.parentPath, entry.name);
            files.set(relative(dir, path).split("\\").join("/"), readFileSync(path));
        }
    }
    return files;
}

function hashLines(files: Map<string, Buffer>): string {
    const lines = [...files]
        .sort(([a], [b]) => (a < b ? -1 : 1))
        .map(([path, contents]) => `${createHash("sha256").update(contents).digest("hex")}  ${path}`);
    return `${lines.join("\n")}\n`;
}

/** Returns why the build output does not contain the Window artifacts a case configures. */
function missingArtifacts(files: Map<string, Buffer>, options: Case): string | undefined {
    const paths = [...files.keys()];
    if (!paths.some((path) => path.startsWith("dist/") && path !== "dist/pack.mcmeta")) {
        return "dist contains nothing besides pack.mcmeta";
    }
    if (
        options.kotlinPackage !== undefined &&
        !paths.some((path) => path.startsWith(`${kotlinDir}/`) && path.endsWith(".kt"))
    ) {
        return "Kotlin is configured but no Kotlin files were produced";
    }
    return undefined;
}

function expectedMismatch(expected: string, actual: string): string {
    const parse = (text: string) =>
        new Map(
            text
                .split("\n")
                .filter(Boolean)
                .map((line) => [line.slice(66), line.slice(0, 64)]),
        );
    const want = parse(expected);
    const got = parse(actual);
    for (const path of [...new Set([...want.keys(), ...got.keys()])].sort()) {
        if (!want.has(path)) {
            return `${path}: not in expected.sha256`;
        }
        if (!got.has(path)) {
            return `${path}: in expected.sha256 but not produced`;
        }
        if (want.get(path) !== got.get(path)) {
            return `${path}: hash differs from expected.sha256`;
        }
    }
    return "expected.sha256 differs";
}

function runCase(name: string): string | undefined {
    const caseDir = join(goldenDir, name);
    const options: Case = JSON.parse(readFileSync(join(caseDir, "case.json"), "utf8"));
    const ts = join(buildDir, name, "ts");
    rmSync(join(buildDir, name), { recursive: true, force: true });
    tsProject(caseDir, ts, options);
    build(ts);

    const outputs = ["dist", kotlinDir];
    const files = new Map(outputs.flatMap((sub) => [...readTree(ts, sub)]));
    if (files.size === 0) {
        return "the build produced no files";
    }
    const missing = missingArtifacts(files, options);
    if (missing !== undefined) {
        return missing;
    }
    const actual = hashLines(files);
    const expectedPath = join(caseDir, "expected.sha256");
    if (update) {
        writeFileSync(expectedPath, actual);
    } else if (!existsSync(expectedPath)) {
        return "expected.sha256 is missing (run with --update to create it)";
    } else {
        const expected = readFileSync(expectedPath, "utf8");
        if (expected !== actual) {
            return expectedMismatch(expected, actual);
        }
    }
    return undefined;
}

const args = process.argv.slice(2);
const update = args.includes("--update");
const requested = args.filter((arg) => arg !== "--update");
const cases = readdirSync(goldenDir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory() && (requested.length === 0 || requested.includes(entry.name)))
    .map((entry) => entry.name)
    .sort();
if (cases.length === 0) {
    throw new Error("no golden cases selected");
}

let failed = false;
for (const name of cases) {
    try {
        const difference = runCase(name);
        console.log(
            `${difference === undefined ? "ok  " : "FAIL"} ${name}${difference === undefined ? "" : `: ${difference}`}`,
        );
        failed ||= difference !== undefined;
    } catch (error) {
        console.log(`FAIL ${name}: ${error instanceof Error ? error.message : String(error)}`);
        failed = true;
    }
}
process.exitCode = failed ? 1 : 0;
