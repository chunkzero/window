#!/usr/bin/env bash
set -euo pipefail
: "${RELEASE_VERSION:?Set the coordinated Window version}"
: "${GITHUB_SHA:?Set the source commit}"
mkdir -p dist
python3 - <<'PY'
import json, os
from pathlib import Path
manifest = Path('plugin/rpp.json')
value = json.loads(manifest.read_text())
value['version'] = os.environ['RELEASE_VERSION']
value['description'] = 'Compiles UIs into resource-pack assets and Kotlin bindings. Source commit: ' + os.environ['GITHUB_SHA']
manifest.write_text(json.dumps(value, indent=2) + '\n')
Path('plugin/release.json').write_text(json.dumps({'version': os.environ['RELEASE_VERSION'], 'source': os.environ['GITHUB_SHA']}) + '\n')
PY
repository=$(pwd)
(cd plugin && rpp plugin pack --json --out "$repository/dist") > dist/packed.json
# Every published JVM library is staged with its POM and transitive Window dependencies.
jvm/gradlew -p jvm --no-daemon \
  -PwindowVersion="$RELEASE_VERSION" \
  :runtime:publish :diagnostics-protocol:publish :minestom-diagnostics:publish
find jvm/build/staging-deploy -name 'maven-metadata.xml*' -delete
name="window-$RELEASE_VERSION-maven"
mkdir -p "build/release/$name"
cp -R jvm/build/staging-deploy/. "build/release/$name/"
cp plugin/release.json "build/release/$name/"
python3 - <<'PYTHON'
import gzip, os, tarfile
from pathlib import Path
name = f"window-{os.environ['RELEASE_VERSION']}-maven"
root = Path('build/release') / name
with open(f'dist/{name}.tar.gz', 'wb') as output, gzip.GzipFile(filename='', mode='wb', fileobj=output, mtime=0) as compressed, tarfile.open(fileobj=compressed, mode='w') as archive:
    for path in sorted(root.rglob('*')):
        if not path.is_file():
            continue
        entry = archive.gettarinfo(path, arcname=path.relative_to(root.parent))
        entry.uid = entry.gid = entry.mtime = 0
        entry.uname = entry.gname = ''
        with path.open('rb') as source:
            archive.addfile(entry, source)
PYTHON
cp plugin/release.json dist/release.json
(cd dist && sha256sum "$name.tar.gz" > "$name.tar.gz.sha256" \
  && sha256sum release.json > release.json.sha256 \
  && sha256sum packed.json > packed.json.sha256)
