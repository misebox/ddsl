export type Lang = "en" | "ja";

/** 先頭が既定の言語で、サイトのルートに出る。 */
export const LANGS = ["en", "ja"] as const satisfies readonly Lang[];

export const DEFAULT_LANG: Lang = LANGS[0];

export const LANG_LABEL: Record<Lang, string> = {
  en: "English",
  ja: "日本語",
};

export function isLang(value: string | undefined): value is Lang {
  return LANGS.some((l) => l === value);
}

/**
 * 表示中の言語。1ページ1言語の静的サイトなので、読み込み時に一度決まって
 * あとは変わらない。props で持ち回るより、ここに置く方が読みやすい。
 */
let current: Lang = DEFAULT_LANG;

export function setLang(value: Lang): void {
  current = value;
}

export function lang(): Lang {
  return current;
}
