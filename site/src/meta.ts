/**
 * HTML の <head> に入るもの。ビルド時に Node 側からも読むので、
 * JSX を持たない素の TypeScript にしてある。
 */
import type { Lang } from "./lang.ts";
import type { PageId } from "./pages.ts";

export const DESCRIPTION: Record<Lang, string> = {
  en: "A schema definition language that compiles to SQL DDL. Name the nouns, and table names, foreign keys, indexes and comments follow.",
  ja: "SQL の DDL にコンパイルするスキーマ定義言語。名詞を決めれば、テーブル名・外部キー・インデックス・コメントはそこから決まる。",
};

export const TITLE: Record<Lang, Record<PageId, string>> = {
  en: {
    index: "NounSQL — a DSL for Database Schema Design",
    guide: "Guide — NounSQL",
    spec: "Specification — NounSQL",
    samples: "Samples — NounSQL",
    playground: "Playground — NounSQL",
    tooling: "How it works — NounSQL",
  },
  ja: {
    index: "NounSQL — データベーススキーマ設計のための DSL",
    guide: "ガイド — NounSQL",
    spec: "仕様 — NounSQL",
    samples: "サンプル — NounSQL",
    playground: "プレイグラウンド — NounSQL",
    tooling: "仕組み — NounSQL",
  },
};
