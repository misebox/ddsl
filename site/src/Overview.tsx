import { CodeTabs } from "./CodeTabs";
import { readExample } from "./content";

/** 概要。何を解くのかだけを書く。使い方は使い方のページに置く。 */
export function Overview() {
  return (
    <>
      <h1>NounSQL</h1>

      <p class="lede">
        <strong>NounSQL is a DSL for Database Schema Design.</strong>
      </p>

      <p>
        スキーマを名詞と規約から組み立てて、PostgreSQL の DDL を出す。
        テーブル名の複数形、FK 列の名前と型、索引の名前は書かない。
      </p>

      <CodeTabs source={readExample("minimal.nsql")} />

      <h2 id="何を解くか">何を解くか</h2>

      <p>
        DDL を直接書くと、同じ列定義・同じ制約・同じ命名規則が全テーブルに散らばる。
        NounSQL はそれを4つの側面から畳む。
      </p>

      <h3 id="規約を1箇所に集める">規約を1箇所に集める</h3>
      <p>
        主キー、作成日時、更新日時のような「どのテーブルにもあるもの」は <code>mixin</code> に置く。
        テーブル定義には、そのテーブルにしか無いものだけが残る。
        規約を変えるときに触るのは1箇所になる。
      </p>

      <h3 id="命名の揺れを無くす">命名の揺れを無くす</h3>
      <p>
        テーブル名・FK 列名・索引名の作り方を <code>naming</code> に書く。
        人によって <code>idx_</code> だったり <code>ix_</code> だったり、
        <code>user_id</code> だったり <code>userId</code> だったりする揺れが、
        レビューではなく定義で決まる。
      </p>

      <h3 id="概念の混同を防ぐ">概念の混同を防ぐ</h3>
      <p>
        影響が及ぶ範囲ごとに構文を分けてある。
        名前を見た時点で何が生成されるかが判るので、中身を読み直さなくて済む。
      </p>
      <table>
        <thead>
          <tr>
            <th>構文</th>
            <th>及ぶ範囲</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>
              <code>mixin</code>
            </td>
            <td>1テーブルの中</td>
          </tr>
          <tr>
            <td>
              <code>belongs_to</code> / <code>associate</code>
            </td>
            <td>2テーブルの間</td>
          </tr>
          <tr>
            <td>
              <code>blueprint</code>
            </td>
            <td>3テーブル以上のまとまり</td>
          </tr>
        </tbody>
      </table>

      <h3 id="用語をスキーマと同じ場所に置く">用語をスキーマと同じ場所に置く</h3>
      <p>
        <code>nouns</code> は一見ただの単複変換表だが、2つの問題を同時に引き受けている。
      </p>
      <p>
        ひとつは<strong>不規則変化</strong>。<code>person</code> → <code>people</code>、
        <code>child</code> → <code>children</code> は機械的な規則では出せない。
        辞書に書けば、変換規則がコメントやコミットログではなく定義そのものとして残る。
      </p>
      <p>
        もうひとつは<strong>用語集</strong>。同じ行に説明を書かせているので、
        「このテーブルは業務上何を指すか」がスキーマ定義と同じ場所に集まる。
        別のドキュメントに置けば必ず陳腐化するものが、
        スキーマを書く行為そのものに含まれる。説明はそのまま DDL の
        <code>COMMENT</code> になる。
      </p>

      <h2 id="次に読むもの">次に読むもの</h2>
      <ul>
        <li>
          <a href="./guide.html">使い方</a> — インストールとコマンド
        </li>
        <li>
          <a href="./samples.html">サンプル</a> — 動く例と生成される DDL
        </li>
        <li>
          <a href="./playground.html">プレイグラウンド</a> — ブラウザで書いて試す
        </li>
        <li>
          <a href="./spec.html">仕様</a> — 構文と解決の規則
        </li>
      </ul>
    </>
  );
}
