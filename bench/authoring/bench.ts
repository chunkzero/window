// Compares `rpp build` times for the same UI written with JSX (`jsx/`) and with the function-style API (`fn/`).
// Each style is built at 1x and at `--scale` (renamed copies of every window), cold, as a no-op cache replay, and
// after an edit to every window source. Both styles must produce identical packs and Kotlin bindings, apart from
// source labels.
// Usage: node bench/authoring/bench.ts [--runs 21] [--scale 10] [--scenarios cold,noop,edit] [--cold-wasm].
// `--cold-wasm` adds a cold scenario that also empties the compiled wasm cache. Builds always use the
// bench-owned RPP_CACHE_DIR build/bench/rpp-cache, so the user-wide cache is never touched. Set RPP to use a
// specific rpp binary.
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { cpSync, existsSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { delimiter, join, relative, resolve } from "node:path";
import { parseArgs } from "node:util";

type Style = "jsx" | "fn";
type Metric = "wall" | "resolve" | "build";
type Sample = Record<Metric, number>;
interface Stats {
    p10: number;
    median: number;
    p90: number;
}

const here = import.meta.dirname;
const root = resolve(here, "../..");
const out = join(root, "build", "bench", "authoring");
const rpp = which(process.env["RPP"] ?? "rpp");
const cacheDir = join(root, "build", "bench", "rpp-cache");
process.env["RPP_CACHE_DIR"] = cacheDir;
const { values } = parseArgs({
    options: {
        runs: { type: "string", default: "21" },
        scale: { type: "string", default: "10" },
        scenarios: { type: "string", default: "cold,noop,edit" },
        "cold-wasm": { type: "boolean", default: false },
    },
});
const runs = Number(values.runs);
const scale = Number(values.scale);
if (!Number.isInteger(runs) || runs < 1 || !Number.isInteger(scale) || scale < 1) {
    throw new Error("--runs and --scale must be positive integers");
}

// Window names to suffix in each copy. Narrow patterns keep same-named controls (the forge button) unchanged.
const RENAMES: Record<Style, [RegExp, string][]> = {
    jsx: [
        [/name="(forge)"\s+container/g, 'name="$1$$" container'],
        [/name="(forge_progress|shop|status_[a-z_]+)"/g, 'name="$1$$"'],
    ],
    fn: [
        [/name:\s*"(forge|forge_progress|shop)"/g, 'name: "$1$$"'],
        [/pinned\(\s*"(status_[a-z_]+)"/g, 'pinned("$1$$"'],
    ],
};

function windowSources(dir: string): string[] {
    const win = join(dir, "src", "window");
    return readdirSync(win)
        .filter((name) => /\.tsx?$/.test(name) && name !== "sprites.ts")
        .map((name) => join(win, name));
}

function project(style: Style, copies: number): string {
    const dir = join(out, `${style}${copies}`);
    rmSync(dir, { recursive: true, force: true });
    cpSync(join(here, style), dir, { recursive: true });
    writeFileSync(join(dir, "rpp.json"), JSON.stringify({ dependencies: { window: `path:${join(root, "plugin")}` } }));
    for (const path of windowSources(dir)) {
        const source = readFileSync(path, "utf8");
        for (let i = 1; i < copies; i++) {
            const copy = RENAMES[style].reduce(
                (text, [from, to]) => text.replace(from, to.replace("$$", `_c${i}`)),
                source,
            );
            writeFileSync(path.replace(/(\.tsx?)$/, `_c${i}$1`), copy);
        }
    }
    return dir;
}

function clean(dir: string): void {
    for (const sub of [".rpp", "dist", "kotlin-out"]) {
        rmSync(join(dir, sub), { recursive: true, force: true });
    }
}

let rev = 0;
/** Rewrites a trailing `export const rev = N;` in every window source; comment-only edits replay from cache. */
function edit(dir: string, next = ++rev): void {
    for (const path of windowSources(dir)) {
        const source = readFileSync(path, "utf8").replace(/\nexport const rev = \d+;\n$/, "\n");
        writeFileSync(path, next === 0 ? source : `${source}export const rev = ${next};\n`);
    }
}

function ms(text: string | undefined): number {
    const match = /^([\d.]+)(ms|s|µs|us)$/.exec(text ?? "");
    if (match === null) {
        return NaN;
    }
    const value = Number(match[1]);
    return match[2] === "s" ? value * 1000 : match[2] === "ms" ? value : value / 1000;
}

function build(dir: string, generated: number): Sample {
    const start = process.hrtime.bigint();
    const result = spawnSync(rpp, ["build", "--no-squash"], { cwd: dir, encoding: "utf8" });
    const wall = Number(process.hrtime.bigint() - start) / 1e6;
    const log = result.stdout + result.stderr;
    if (result.status !== 0) {
        throw new Error(`rpp build failed in ${relative(root, dir)}:\n${result.error ?? log}`);
    }
    if (Number(/generated (\d+)/.exec(log)?.[1]) !== generated) {
        throw new Error(`${relative(root, dir)}: expected generated ${generated}:\n${log}`);
    }
    return { wall, resolve: ms(/resolved in (\S+)/.exec(log)?.[1]), build: ms(/finished in (\S+)/.exec(log)?.[1]) };
}

/** File contents without source labels, which name each style's own constructors (`tab` vs `choice`). */
function comparable(path: string): Buffer | string {
    const bytes = readFileSync(path);
    if (!/\.(json|kt)$/.test(path)) {
        return bytes;
    }
    return bytes
        .toString("utf8")
        .replace(/"source":"[^"]*"/g, "")
        .replace(/source = "[^"]*"/g, "")
        .replace(/"pack_fingerprint":\{[^}]*\}/g, "");
}

function digest(dir: string): string {
    const hash = createHash("sha256");
    for (const sub of ["dist", "kotlin-out"]) {
        const top = join(dir, sub);
        if (!existsSync(top)) {
            continue;
        }
        const files = readdirSync(top, { recursive: true, withFileTypes: true })
            .filter((entry) => entry.isFile())
            .map((entry) => join(entry.parentPath, entry.name))
            .sort();
        for (const path of files) {
            hash.update(relative(dir, path)).update(comparable(path));
        }
    }
    return hash.digest("hex");
}

function stats(samples: number[]): Stats {
    const sorted = [...samples].sort((a, b) => a - b);
    const at = (p: number): number => {
        const i = (sorted.length - 1) * p;
        const lo = Math.floor(i);
        return sorted[lo]! + (sorted[Math.ceil(i)]! - sorted[lo]!) * (i - lo);
    };
    return { p10: at(0.1), median: at(0.5), p90: at(0.9) };
}

interface Scenario {
    prepare: (dir: string) => void;
    prime: boolean;
    generated: number;
}

const SCENARIOS: Record<string, Scenario> = {
    cold: { prepare: clean, prime: false, generated: 1 },
    "cold-wasm": {
        prepare: (dir) => {
            clean(dir);
            rmSync(join(cacheDir, "wasmtime"), { recursive: true, force: true });
        },
        prime: false,
        generated: 1,
    },
    noop: { prepare: () => {}, prime: true, generated: 0 },
    edit: { prepare: (dir) => edit(dir), prime: true, generated: 1 },
};

const selected = [...values.scenarios.split(","), ...(values["cold-wasm"] ? ["cold-wasm"] : [])];
for (const name of selected) {
    if (!(name in SCENARIOS)) {
        throw new Error(`unknown scenario ${name}; expected ${Object.keys(SCENARIOS).join(", ")}`);
    }
}

function which(command: string): string {
    if (command.includes("/")) {
        return resolve(command);
    }
    const dirs = (process.env["PATH"] ?? "").split(delimiter);
    return dirs.map((dir) => resolve(dir, command)).find((path) => existsSync(path)) ?? command;
}

const wasm = join(root, "plugin", "window.wasm");
const f = (n: number): string => n.toFixed(1);
console.log(`rpp: ${rpp} (${spawnSync(rpp, ["--version"], { encoding: "utf8" }).stdout.trim()})`);
console.log(`window.wasm sha256: ${createHash("sha256").update(readFileSync(wasm)).digest("hex")}`);
console.log(`RPP_CACHE_DIR: ${cacheDir}, runs per cell: ${runs}`);
console.log();
console.log("| Scale | Scenario | Style | Wall p10 / median / p90 (ms) | Resolve median | Build p10 / median / p90 |");
console.log("|---|---|---|---|---|---|");
for (const copies of [1, scale]) {
    const dirs = { jsx: project("jsx", copies), fn: project("fn", copies) };
    for (const dir of Object.values(dirs)) {
        clean(dir);
        build(dir, 1);
    }
    if (digest(dirs.jsx) !== digest(dirs.fn)) {
        throw new Error(`${copies}x: JSX and function-style outputs differ; diff -r ${dirs.jsx} ${dirs.fn}`);
    }
    for (const scenario of selected) {
        const config = SCENARIOS[scenario]!;
        const samples: Record<Style, Sample[]> = { jsx: [], fn: [] };
        for (const dir of Object.values(dirs)) {
            if (config.prime) {
                clean(dir);
                build(dir, 1);
            }
        }
        for (let i = 0; i < runs; i++) {
            for (const style of (i % 2 === 0 ? ["jsx", "fn"] : ["fn", "jsx"]) as Style[]) {
                config.prepare(dirs[style]);
                samples[style].push(build(dirs[style], config.generated));
            }
        }
        for (const dir of Object.values(dirs)) {
            edit(dir, 0);
        }
        for (const style of ["jsx", "fn"] as Style[]) {
            const metric = (key: Metric): Stats => stats(samples[style].map((sample) => sample[key]));
            const [wall, resolveTime, buildTime] = [metric("wall"), metric("resolve"), metric("build")];
            console.log(
                `| ${copies}x | ${scenario} | ${style} | ${f(wall.p10)} / ${f(wall.median)} / ${f(wall.p90)} | ` +
                    `${f(resolveTime.median)} | ${f(buildTime.p10)} / ${f(buildTime.median)} / ${f(buildTime.p90)} |`,
            );
        }
    }
}
