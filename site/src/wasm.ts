/**
 * コンパイラ本体。CLI と同じものを WebAssembly に落としたもの。
 *
 * ドキュメントのコードブロックの色付けにも使うので、
 * プレイグラウンド以外のページでも読み込む。
 */
import { createSignal } from "solid-js";
import init, {
  compile,
  dialects,
  highlight_html,
  highlight_sql_html,
} from "nounsql";

export type Wasm = {
  compile: typeof compile;
  dialects: typeof dialects;
  highlightSource: typeof highlight_html;
  highlightSql: typeof highlight_sql_html;
};

const [wasm, setWasm] = createSignal<Wasm | undefined>();
const [failed, setFailed] = createSignal(false);

export { wasm, failed };

let started: Promise<void> | undefined;

/** 何度呼んでも読み込みは1度だけ。 */
export function loadWasm(): Promise<void> {
  started ??= init()
    .then(() => {
      setWasm({
        compile,
        dialects,
        highlightSource: highlight_html,
        highlightSql: highlight_sql_html,
      });
    })
    .catch(() => {
      setFailed(true);
    });
  return started;
}
