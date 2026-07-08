# the infra graph of `exemplar`, DECLARED — surfaces, stores, seams, credentials
# (names only), authorities, cadences. Judged laws: cors-covers-origins,
# secrets-census, build-command-is-derived (against live API reads); the
# rest lives on the register floor (`exemplar.infra.register`) — declared but
# unjudgeable, disclosed as exactly that. Regenerate with
# `cargo run --example freeze_infra`.

surface app (prod) — origins: https://app.example; build: npm ci && npm run build
surface app (preview) — origins: https://preview.app.example; build: npm ci && npm run build
store intake — roots:
  test/ means ephemeral(24h)
  drafts/ means drafts
  v1/ means locked
seam: app (prod) presigns PUT into intake under v1/
seam: app (preview) presigns PUT into intake under test/
seam: app (prod) reads intake
seam: app (preview) reads intake
credentials of app (prod): OAUTH_CLIENT_SECRET, STORE_ACCESS_KEY_ID, STORE_SECRET_ACCESS_KEY
credentials of app (preview): OAUTH_CLIENT_SECRET, STORE_ACCESS_KEY_ID, STORE_SECRET_ACCESS_KEY
authority test-token mint — mints test tokens (operator org membership — a declared holdover); reaches: test/
cadence reaper — daily — deletes ephemeral roots past their TTL
cadence post-deploy probe — every deploy — exercises the presign seams end to end
