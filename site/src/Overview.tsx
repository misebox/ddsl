import { CodeTabs } from "./CodeTabs";
import { readExample } from "./content";

export function Overview() {
  return (
    <>
      <section class="hero">
        <h1>Write the conventions once.</h1>

        <p class="hero-lede">
          Mixins, naming rules and a dictionary of nouns settle the repetitive parts of a schema.
          What stays in a table definition is what makes that table different.
        </p>

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
              <code>belongs_to</code>
            </td>
            <td>from one table to another</td>
          </tr>
          <tr>
            <td>
              <code>associate</code>
            </td>
            <td>a table placed between two</td>
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
