## line-diff-wasm

The goal of this project is to generate a diff between two texts which can
be used to render an editor's "gutter" markers. It generates "add", "remove", and "modify" markers
similar to those rendered in most modern IDEs for git diffs.

The output format is a crazy byte array which can be passed across the wasm boundary.

Example interpretation and usage (esbuild):

```ts
import { default as wasmbin } from "line-diff-wasm/line_diff_wasm_bg.wasm";
import init, { line_diff } from "line-diff-wasm";

export enum EditorLineDecorationKind {
  ADD,
  CHANGE,
  DELETE,
}

export interface EditorLineDecoration {
  lineStart: number;
  lineEnd: number;
  kind: EditorLineDecorationKind;
}

init(wasmbin);

// load wasm and parse insane uint8array into the usable format
export default function lineDiff(
  oldText: string,
  newText: string
): EditorLineDecoration[] {
  const magicNumbers = line_diff(oldText, newText);
  const view = new DataView(magicNumbers.buffer);
  const results: EditorLineDecoration[] = [];
  for (let i = 0; i < magicNumbers.length; i += 9) {
    const lineStart = view.getUint32(i, false);
    const lineEnd = view.getUint32(i + 4, false);
    const kindInt = view.getUint8(i + 8);
    let kind = EditorLineDecorationKind.ADD;
    switch (kindInt) {
      case 1:
        kind = EditorLineDecorationKind.ADD;
        break;
      case 2:
        kind = EditorLineDecorationKind.DELETE;
        break;
      case 3:
        kind = EditorLineDecorationKind.CHANGE;
        break;
    }
    results.push({
      lineStart,
      lineEnd,
      kind,
    });
  }
  return results;
}
```
