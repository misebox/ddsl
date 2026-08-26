import { For } from "solid-js";
import { CodeTabs } from "./CodeTabs";
import { readExample } from "./content";

/**
 * One row of vocabulary, and every name the compiler builds from it.
 *
 * These are the real names `nounsql` emits for the schema in the
 * example below — not an illustration.
 */
const DERIVED = [
  { name: "users", from: "table name" },
  { name: "user_id", from: "foreign key on posts" },
  { name: "idx_posts_user_id", from: "its index" },
  { name: "posts_user_id_fkey", from: "the constraint" },
  { name: "'A person who signs in'", from: "COMMENT ON TABLE" },
];

export function Overview() {
  return (
    <>
      <section class="hero">
        <p class="hero-eyebrow">Data Definition Schema Language</p>

        <h1>Name the nouns. The rest of the names follow.</h1>

        <p class="hero-lede">
          A schema language that compiles to PostgreSQL DDL. You write the vocabulary and the
          conventions once; table names, foreign keys, indexes and comments come out of them.
        </p>

        <div class="derive">
          <p class="derive-label">One line in nouns</p>

          <div class="derive-seed">
            <b>user users</b>
            <span>"A person who signs in"</span>
          </div>

          <div class="derive-out">
            <For each={DERIVED}>
              {(row) => (
                <p class="derive-row">
                  <b>{row.name}</b>
                  <span>{row.from}</span>
                </p>
              )}
            </For>
          </div>
        </div>

        <div class="hero-actions">
          <a class="button" href="./playground.html">
            Try it in the browser
          </a>
          <a class="button is-quiet" href="./guide.html">
            Install
          </a>
        </div>
      </section>

      <CodeTabs source={readExample("minimal.nsql")} />

      <h2 id="what-it-removes">What it removes</h2>

      <p>
        Writing DDL by hand spreads the same column definitions, the same constraints and the same
        naming habits across every table. NounSQL folds that in four places.
      </p>

      <h3 id="conventions-live-in-one-place">Conventions live in one place</h3>
      <p>
        Primary keys, timestamps, anything every table carries — put it in a <code>mixin</code>. What
        stays in a table definition is what makes that table different. Changing a convention means
        editing one block.
      </p>

      <h3 id="names-stop-drifting">Names stop drifting</h3>
      <p>
        <code>naming</code> holds the rules for table names, foreign key columns and index names. The
        difference between <code>idx_</code> and <code>ix_</code>, between <code>user_id</code> and{" "}
        <code>userId</code>, is settled by a definition instead of by review comments.
      </p>

      <h3 id="scope-is-visible-in-the-syntax">Scope is visible in the syntax</h3>
      <p>
        Three constructs, split by how far their effect reaches. You can tell what a name generates
        without opening it.
      </p>
      <table>
        <thead>
          <tr>
            <th>Construct</th>
            <th>Reaches</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>
              <code>mixin</code>
            </td>
            <td>inside one table</td>
          </tr>
          <tr>
            <td>
              <code>belongs_to</code>, <code>associate</code>
            </td>
            <td>between two tables</td>
          </tr>
          <tr>
            <td>
              <code>blueprint</code>
            </td>
            <td>a group of three or more</td>
          </tr>
        </tbody>
      </table>

      <h3 id="the-glossary-is-a-by-product">The glossary is a by-product</h3>
      <p>
        <code>nouns</code> looks like a plural table. It does two jobs.
      </p>
      <p>
        It makes irregular forms part of the definition. <code>person</code> → <code>people</code>,{" "}
        <code>child</code> → <code>children</code>: no rule produces those. Written down, the
        conversion lives in the source instead of in a comment or a commit message.
      </p>
      <p>
        And it collects the glossary. The third column says what a table means to the business, in
        the same file as the schema, and ends up in the DDL as a <code>COMMENT</code>. Kept anywhere
        else, that description goes stale.
      </p>

      <h2 id="next">Next</h2>
      <ul>
        <li>
          <a href="./guide.html">Guide</a> — install and run the compiler
        </li>
        <li>
          <a href="./samples.html">Samples</a> — working schemas and the DDL they produce
        </li>
        <li>
          <a href="./playground.html">Playground</a> — edit and compile in the browser
        </li>
        <li>
          <a href="./spec.html">Specification</a> — syntax and resolution rules
        </li>
      </ul>
    </>
  );
}
