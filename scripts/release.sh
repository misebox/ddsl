#!/usr/bin/env bash
#
# リリースの準備をする。
#
#   scripts/release.sh 0.2.0          バージョンを上げ、コミットとタグを作る
#   scripts/release.sh 0.2.0 --check  検査だけ行い、何も書き換えない
#
# タグを push すると .github/workflows/release.yml が動き、
# バイナリのビルド・GitHub Release の作成・crates.io への公開を行う。
# push はこのスクリプトでは行わない。
set -euo pipefail

cd "$(dirname "$0")/.."

# crates.io へは依存の順に公開する。
CRATES=(ddsl-core ddsl-lsp ddsl)

usage() {
  sed -n '2,9p' "$0" | sed 's/^# \{0,1\}//'
  exit 1
}

die() {
  echo "error: $*" >&2
  exit 1
}

step() {
  echo
  echo "==> $*"
}

# GNU sed と BSD sed の -i の差を吸収する。
replace_in_file() {
  local pattern=$1 file=$2 tmp
  tmp=$(mktemp)
  sed "$pattern" "$file" > "$tmp"
  mv "$tmp" "$file"
}

[ $# -ge 1 ] || usage
VERSION=$1
CHECK_ONLY=${2:-}
TAG="v$VERSION"

[[ "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]] \
  || die "バージョンが semver ではない: $VERSION"

step "作業ツリーの確認"
[ -z "$(git status --porcelain)" ] || die "コミットされていない変更がある"
git rev-parse "$TAG" >/dev/null 2>&1 && die "タグ $TAG は既にある"
echo "ok"

step "フォーマット"
cargo fmt --check

step "lint"
cargo clippy --all-targets -- -D warnings

step "テスト"
cargo test

step "サンプルの DDL が最新か"
cargo run -q -p ddsl -- sql examples/sample.ddsl > examples/sample.sql
git diff --quiet examples/sample.sql \
  || die "examples/sample.sql が古い。生成し直してコミットする"
echo "ok"

step "ドキュメントサイトが生成できるか"
cargo run -q -p ddsl-site >/dev/null
echo "ok"

step "パッケージの中身"
for crate in "${CRATES[@]}"; do
  count=$(cargo package --list -p "$crate" | wc -l | tr -d ' ')
  echo "  $crate: $count files"
done

# 依存が未公開だと --dry-run の検証ビルドが通らないので、
# レジストリに載っている場合だけ実行する。
step "publish の予行"
if curl -sf -o /dev/null "https://crates.io/api/v1/crates/ddsl-core"; then
  for crate in "${CRATES[@]}"; do
    echo "  $crate"
    cargo publish --dry-run -p "$crate" >/dev/null
  done
else
  echo "  ddsl-core が未公開のため package のみ検証する"
  cargo package -p ddsl-core >/dev/null
fi

if [ "$CHECK_ONLY" = "--check" ]; then
  step "検査のみ完了。何も書き換えていない"
  exit 0
fi

step "バージョンを $VERSION にする"
replace_in_file "s/^version = \"[^\"]*\"$/version = \"$VERSION\"/" Cargo.toml
replace_in_file \
  "s|^ddsl-core = { path = \"crates/ddsl-core\", version = \"[^\"]*\" }|ddsl-core = { path = \"crates/ddsl-core\", version = \"$VERSION\" }|" \
  Cargo.toml
cargo update --workspace --quiet
grep -n "^version = " Cargo.toml
grep -n "^ddsl-core = " Cargo.toml

step "コミットとタグ"
git add Cargo.toml Cargo.lock
git commit -m "Release $TAG"
git tag -a "$TAG" -m "$TAG"

cat <<EOF

準備ができた。内容を確認してから push する。

  git show HEAD
  git push origin HEAD
  git push origin $TAG

タグを push すると CI が動き、バイナリのビルド・GitHub Release の作成・
crates.io への公開が行われる。crates.io への公開は取り消せない。

取り消す場合:

  git tag -d $TAG
  git reset --soft HEAD~1
EOF
