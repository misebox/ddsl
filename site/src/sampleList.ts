/** Shared by the samples page and the playground. The files live in examples/. */
export type Sample = {
  readonly file: string;
  readonly label: string;
  readonly summary: string;
};

export const SAMPLES: readonly Sample[] = [
  {
    file: "minimal.nsql",
    label: "Minimal",
    summary: "Register the nouns, put shared columns in a mixin, connect tables with belongs_to.",
  },
  {
    file: "mixin.nsql",
    label: "Mixins",
    summary: "Keep conventions in one place and write only what makes each table different.",
  },
  {
    file: "relations.nsql",
    label: "Relations",
    summary: "Two references to the same table, a self reference, and a join table with columns.",
  },
  {
    file: "blueprint.nsql",
    label: "Blueprints",
    summary: "Three tables that only mean something together, held under one name.",
  },
  {
    file: "naming.nsql",
    label: "Naming",
    summary: "Where table names, foreign key columns and index names come from.",
  },
  {
    file: "shop.nsql",
    label: "A real schema",
    summary: "Ten tables of an online shop. On top of conventions, only the differences remain.",
  },
  {
    file: "sample.nsql",
    label: "Everything",
    summary: "Every construct that appears in the specification.",
  },
];
