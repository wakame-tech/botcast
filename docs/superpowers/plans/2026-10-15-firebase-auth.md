# Firebase Auth (Google) ログイン Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** botcast-cms と botcast のメール + パスワード認証を、Firebase Auth の ID トークンを CMS JWT に交換する方式に置き換える。

**Architecture:** クライアントは Firebase JS SDK の `signInWithPopup(GoogleAuthProvider)` で ID トークンを得て `POST /t/{t}/auth/firebase` に送る。CMS は securetoken の JWKS（キャッシュ付き）で RS256 / `iss` / `aud` / `exp` を検証し、`user` を `uid` で upsert して既存の `issue_jwt` で CMS JWT を返す。以降の認証・所有者・API キーは変えない。

**Tech Stack:** Rust (axum 0.8, jsonwebtoken 9, reqwest 0.12, surrealdb 2.3), React (CMS web: react-router / botcast web: TanStack Router), `firebase` JS SDK

**設計:** `docs/superpowers/specs/2026-10-15-firebase-auth-design.md`

**作業場所:**
- botcast-cms: worktree `~/dev/botcast-cms-firebase`（ブランチ `feat/firebase-auth`、`origin/develop` から）。`~/dev/botcast-cms` 本体には触れない
- botcast: `~/dev/botcast`（ブランチ `docs/firebase-auth-design` に続けてコミット）

**Firebase config（公開値）:** `apiKey: AIzaSyBWgxz2n9U1akZYh7d8-2UmniFnYGVDsW8` / `authDomain: botcast-cms.firebaseapp.com` / `projectId: botcast-cms`

---

## botcast-cms

### Task 1: テスト用 RSA 鍵と ID トークン検証関数

**Files:**
- Create: `cms/crates/api/src/testdata/firebase_test_key.pem`、`cms/crates/api/src/testdata/firebase_test_jwks.json`
- Create: `cms/crates/api/src/firebase.rs`
- Modify: `cms/crates/api/src/main.rs`（`mod firebase;`）

- [ ] **Step 1: テスト専用の鍵と JWKS を作る**

```bash
cd ~/dev/botcast-cms-firebase/cms/crates/api/src && mkdir -p testdata
openssl genrsa -out testdata/firebase_test_key.pem 2048
N=$(openssl rsa -in testdata/firebase_test_key.pem -noout -modulus | cut -d= -f2)
python3 -c "
import base64,json,sys
n=base64.urlsafe_b64encode(bytes.fromhex('$N')).rstrip(b'=').decode()
json.dump({'keys':[{'kty':'RSA','alg':'RS256','use':'sig','kid':'test-kid','n':n,'e':'AQAB'}]},open('testdata/firebase_test_jwks.json','w'))"
```

鍵はテスト専用（本番では使わない）。コミット時に secret scan で止まったら、ファイル先頭にテスト専用である旨のコメントは入れられない（PEM）ので、scan の除外設定に `cms/crates/api/src/testdata/` を足す。

- [ ] **Step 2: 失敗するテストを書く** — `firebase.rs`

```rust
// @specre 01KQS7YE9H5SMQB6BBNKNSVW8T
//! Firebase Auth の ID トークンを検証する。
//! 鍵は securetoken の JWKS。`iss` / `aud` はプロジェクト ID で決まる。
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header, jwk::JwkSet};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct FirebaseClaims {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum VerifyError {
    /// 署名・クレームが不正
    Invalid,
    /// `kid` が JWKS に無い（鍵のローテーション直後なら取り直せば通る）
    UnknownKey,
}

pub fn verify_id_token(
    token: &str,
    project_id: &str,
    jwks: &JwkSet,
) -> Result<FirebaseClaims, VerifyError> {
    todo!()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use jsonwebtoken::{EncodingKey, Header, encode};
    use serde_json::json;

    pub const PROJECT: &str = "test-project";

    pub fn test_jwks() -> JwkSet {
        serde_json::from_str(include_str!("testdata/firebase_test_jwks.json")).unwrap()
    }

    pub fn sign(claims: serde_json::Value, kid: &str) -> String {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(kid.to_string());
        let key = EncodingKey::from_rsa_pem(include_bytes!("testdata/firebase_test_key.pem")).unwrap();
        encode(&header, &claims, &key).unwrap()
    }

    pub fn valid_claims() -> serde_json::Value {
        let now = chrono::Utc::now().timestamp();
        json!({
            "iss": format!("https://securetoken.google.com/{PROJECT}"),
            "aud": PROJECT,
            "sub": "uid-1",
            "iat": now,
            "exp": now + 3600,
            "email": "a@example.com",
            "name": "A",
        })
    }

    fn with(key: &str, value: serde_json::Value) -> serde_json::Value {
        let mut c = valid_claims();
        c[key] = value;
        c
    }

    #[test]
    fn accepts_valid_token() {
        let got = verify_id_token(&sign(valid_claims(), "test-kid"), PROJECT, &test_jwks());
        assert_eq!(
            got,
            Ok(FirebaseClaims {
                sub: "uid-1".into(),
                email: Some("a@example.com".into()),
                name: Some("A".into()),
            })
        );
    }

    #[test]
    fn rejects_wrong_audience() {
        let t = sign(with("aud", json!("other")), "test-kid");
        assert_eq!(verify_id_token(&t, PROJECT, &test_jwks()), Err(VerifyError::Invalid));
    }

    #[test]
    fn rejects_wrong_issuer() {
        let t = sign(with("iss", json!("https://accounts.google.com")), "test-kid");
        assert_eq!(verify_id_token(&t, PROJECT, &test_jwks()), Err(VerifyError::Invalid));
    }

    #[test]
    fn rejects_expired() {
        let past = chrono::Utc::now().timestamp() - 3600;
        let t = sign(with("exp", json!(past)), "test-kid");
        assert_eq!(verify_id_token(&t, PROJECT, &test_jwks()), Err(VerifyError::Invalid));
    }

    #[test]
    fn rejects_empty_subject() {
        let t = sign(with("sub", json!("")), "test-kid");
        assert_eq!(verify_id_token(&t, PROJECT, &test_jwks()), Err(VerifyError::Invalid));
    }

    #[test]
    fn unknown_kid_is_reported() {
        let t = sign(valid_claims(), "other-kid");
        assert_eq!(verify_id_token(&t, PROJECT, &test_jwks()), Err(VerifyError::UnknownKey));
    }

    #[test]
    fn rejects_hs256() {
        let t = encode(
            &Header { kid: Some("test-kid".into()), ..Header::new(Algorithm::HS256) },
            &valid_claims(),
            &EncodingKey::from_secret(b"x"),
        )
        .unwrap();
        assert_eq!(verify_id_token(&t, PROJECT, &test_jwks()), Err(VerifyError::Invalid));
    }
}
```

`main.rs` に `mod firebase;` を追加。

- [ ] **Step 3: 失敗を確認** — `cd cms && rtk proxy cargo test -p api firebase` → `not yet implemented` で FAIL

- [ ] **Step 4: 実装**

```rust
pub fn verify_id_token(
    token: &str,
    project_id: &str,
    jwks: &JwkSet,
) -> Result<FirebaseClaims, VerifyError> {
    let header = decode_header(token).map_err(|_| VerifyError::Invalid)?;
    if header.alg != Algorithm::RS256 {
        return Err(VerifyError::Invalid);
    }
    let kid = header.kid.ok_or(VerifyError::Invalid)?;
    let jwk = jwks.find(&kid).ok_or(VerifyError::UnknownKey)?;
    let key = DecodingKey::from_jwk(jwk).map_err(|_| VerifyError::Invalid)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[project_id]);
    validation.set_issuer(&[format!("https://securetoken.google.com/{project_id}")]);
    validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);

    let claims = decode::<FirebaseClaims>(token, &key, &validation)
        .map_err(|_| VerifyError::Invalid)?
        .claims;
    if claims.sub.is_empty() {
        return Err(VerifyError::Invalid);
    }
    Ok(claims)
}
```

- [ ] **Step 5: 通ることを確認** — 同じコマンドで 7 件 PASS

- [ ] **Step 6: Commit** — `feat(api): Firebase の ID トークンを検証する`

### Task 2: JWKS を取得・キャッシュする `FirebaseVerifier`

**Files:** Modify `cms/crates/api/src/firebase.rs`

- [ ] **Step 1: 失敗するテスト**（`tests` モジュールに追加）

```rust
    #[test]
    fn parses_max_age() {
        assert_eq!(max_age(Some("public, max-age=19830, must-revalidate")), Some(19830));
        assert_eq!(max_age(Some("no-cache")), None);
        assert_eq!(max_age(None), None);
    }

    #[tokio::test]
    async fn verifier_uses_cached_jwks() {
        let v = FirebaseVerifier::with_jwks(PROJECT, test_jwks());
        let got = v.verify(&sign(valid_claims(), "test-kid")).await.unwrap();
        assert_eq!(got.sub, "uid-1");
    }
```

- [ ] **Step 2: 失敗を確認**（未定義でコンパイルエラー）

- [ ] **Step 3: 実装**（`verify_id_token` の後ろ）

```rust
const DEFAULT_JWKS_URL: &str =
    "https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com";
/// `Cache-Control` に `max-age` が無いときのキャッシュ秒数
const DEFAULT_MAX_AGE: u64 = 3600;

pub struct FirebaseVerifier {
    project_id: String,
    jwks_url: String,
    http: reqwest::Client,
    cache: tokio::sync::RwLock<Option<(JwkSet, std::time::Instant)>>,
}

impl FirebaseVerifier {
    /// `FIREBASE_PROJECT_ID` が無ければ `None`（Firebase ログインは 500 になる）。
    /// `FIREBASE_JWKS_URL` で JWKS の取得先を差し替えられる（ローカルの結合確認用）。
    pub fn from_env() -> Option<Self> {
        let project_id = std::env::var("FIREBASE_PROJECT_ID").ok()?;
        let jwks_url =
            std::env::var("FIREBASE_JWKS_URL").unwrap_or_else(|_| DEFAULT_JWKS_URL.to_string());
        Some(Self {
            project_id,
            jwks_url,
            http: reqwest::Client::new(),
            cache: tokio::sync::RwLock::new(None),
        })
    }

    #[cfg(test)]
    pub fn with_jwks(project_id: &str, jwks: JwkSet) -> Self {
        let until = std::time::Instant::now() + std::time::Duration::from_secs(DEFAULT_MAX_AGE);
        Self {
            project_id: project_id.to_string(),
            jwks_url: DEFAULT_JWKS_URL.to_string(),
            http: reqwest::Client::new(),
            cache: tokio::sync::RwLock::new(Some((jwks, until))),
        }
    }

    pub async fn verify(&self, token: &str) -> Result<FirebaseClaims, AppError> {
        let jwks = self.jwks(false).await?;
        match verify_id_token(token, &self.project_id, &jwks) {
            Err(VerifyError::UnknownKey) => {
                let jwks = self.jwks(true).await?;
                verify_id_token(token, &self.project_id, &jwks).map_err(|_| AppError::Unauthorized)
            }
            other => other.map_err(|_| AppError::Unauthorized),
        }
    }

    async fn jwks(&self, refresh: bool) -> Result<JwkSet, AppError> {
        if !refresh {
            if let Some((jwks, until)) = &*self.cache.read().await {
                if std::time::Instant::now() < *until {
                    return Ok(jwks.clone());
                }
            }
        }
        let res = self
            .http
            .get(&self.jwks_url)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|e| AppError::internal(format!("failed to fetch Firebase JWKS: {e}")))?;
        let secs = max_age(
            res.headers()
                .get(reqwest::header::CACHE_CONTROL)
                .and_then(|v| v.to_str().ok()),
        )
        .unwrap_or(DEFAULT_MAX_AGE);
        let jwks: JwkSet = res
            .json()
            .await
            .map_err(|e| AppError::internal(format!("invalid Firebase JWKS: {e}")))?;
        let until = std::time::Instant::now() + std::time::Duration::from_secs(secs);
        *self.cache.write().await = Some((jwks.clone(), until));
        Ok(jwks)
    }
}

fn max_age(cache_control: Option<&str>) -> Option<u64> {
    cache_control?
        .split(',')
        .find_map(|p| p.trim().strip_prefix("max-age=")?.parse().ok())
}
```

`use crate::error::AppError;` を追加。`AppError::internal` の引数型が `&str` 限定なら `format!` の結果を `&` で渡す。

- [ ] **Step 4: 通ることを確認** — `rtk proxy cargo test -p api firebase` → 9 件 PASS
- [ ] **Step 5: Commit** — `feat(api): Firebase の JWKS をキャッシュして取得する`

### Task 3: `/auth/firebase` ハンドラーとパスワード認証の削除

**Files:**
- Modify: `cms/crates/api/src/auth.rs`、`cms/crates/api/src/main.rs`、`cms/crates/api/src/repository/surrealdb.rs`（`init_auth_schema`）、`cms/crates/api/src/middleware/tenant.rs`（テストの `AppState`）

- [ ] **Step 1: `AppState` に verifier を足す**（`main.rs`）

```rust
pub struct AppState {
    pub dify_sandbox_client: Arc<DifySandboxClient>,
    pub jwt_secret: String,
    pub tenants: Arc<tenant::TenantRegistry>,
    /// `FIREBASE_PROJECT_ID` 未設定なら `None`
    pub firebase: Option<Arc<firebase::FirebaseVerifier>>,
}
```

`main()` で `firebase: firebase::FirebaseVerifier::from_env().map(Arc::new)`、未設定なら `eprintln!("Warning: FIREBASE_PROJECT_ID not set, sign-in is disabled")`。`middleware/tenant.rs` の `test_state()` に `firebase: None`。

- [ ] **Step 2: `auth.rs` を書き換える**

`SignUpRequest` / `SignInRequest` / `sign_up_handler` / `sign_in_handler` / `ChangePasswordRequest` / `change_password_handler` を削除し、以下を追加（`AuthResponse` / `JwtClaims` / `UserRecord` / `issue_jwt` は残す）。

```rust
#[derive(Deserialize)]
pub struct FirebaseSignInRequest {
    pub id_token: String,
}

/// Firebase の ID トークンを検証し、テナントのユーザーを `uid` で作成・更新して CMS の JWT を返す
pub async fn firebase_sign_in_handler(
    State(state): State<AppState>,
    Json(body): Json<FirebaseSignInRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let verifier = state
        .firebase
        .as_ref()
        .ok_or_else(|| AppError::internal("FIREBASE_PROJECT_ID is not set"))?;
    let claims = verifier.verify(&body.id_token).await?;

    let tenant = crate::middleware::tenant::current_tenant()
        .ok_or_else(|| AppError::internal("tenant context is missing"))?;
    let db = tenant.db()?;
    let user_id = upsert_user(db, &claims).await?;

    let token = issue_jwt(user_id, &tenant.id, &state.jwt_secret)?;
    Ok(Json(AuthResponse { token }))
}

async fn upsert_user(
    db: &surrealdb::Surreal<surrealdb::engine::remote::ws::Client>,
    claims: &crate::firebase::FirebaseClaims,
) -> Result<String, AppError> {
    let updated: Option<UserRecord> = db
        .query("UPDATE user SET email = $email, name = $name WHERE uid = $uid RETURN id")
        .bind(("uid", claims.sub.clone()))
        .bind(("email", claims.email.clone()))
        .bind(("name", claims.name.clone()))
        .await
        .map_err(AppError::Database)?
        .take(0)
        .map_err(AppError::Database)?;
    if let Some(user) = updated {
        return Ok(user.id.to_string());
    }

    let created: Option<UserRecord> = db
        .query("CREATE user SET uid = $uid, email = $email, name = $name RETURN id")
        .bind(("uid", claims.sub.clone()))
        .bind(("email", claims.email.clone()))
        .bind(("name", claims.name.clone()))
        .await
        .map_err(AppError::Database)?
        .take(0)
        .map_err(AppError::Database)?;
    created
        .map(|u| u.id.to_string())
        .ok_or_else(|| AppError::internal("Failed to create user"))
}
```

`Surreal<Client>` の型は `tenant.rs` の `db()` の戻り値に合わせる（`use` を確認）。`chrono` / `Duration` など未使用になった import を消す。

- [ ] **Step 3: ルートを差し替える**（`main.rs`）

`auth_routes` を以下にし、`protected_auth_routes` とその `merge` を削除。

```rust
    // Firebase ログインは認証不要。ただしテナントの解決は必要
    let auth_routes = axum::Router::new()
        .route(
            "/t/{tenantId}/auth/firebase",
            axum::routing::post(auth::firebase_sign_in_handler),
        )
        .with_state(state.clone())
        .route_layer(tenant_layer());
```

- [ ] **Step 4: `init_auth_schema` を差し替える**（`repository/surrealdb.rs`）

```rust
pub async fn init_auth_schema(db: &Surreal<Client>) -> anyhow::Result<()> {
    // パスワード認証の名残 (DEFINE ACCESS / pass / email の UNIQUE) を消し、Firebase の uid で引く
    db.query(
        "
        REMOVE ACCESS IF EXISTS user ON DATABASE;
        REMOVE INDEX IF EXISTS idx_user_email ON user;
        REMOVE FIELD IF EXISTS pass ON user;
        DEFINE TABLE IF NOT EXISTS user SCHEMAFULL;
        DEFINE FIELD OVERWRITE uid ON user TYPE string;
        DEFINE FIELD OVERWRITE email ON user TYPE option<string>;
        DEFINE FIELD OVERWRITE name ON user TYPE option<string>;
        DEFINE INDEX IF NOT EXISTS idx_user_uid ON user FIELDS uid UNIQUE;
        ",
    )
    .await?
    .check()?;
    println!("Auth schema initialized");
    Ok(())
}
```

- [ ] **Step 5: ビルド・テスト** — `cd cms && cargo check -p api && rtk proxy cargo test -p api` → 全 PASS。`cargo clippy -p api` で新規の警告が無いこと

- [ ] **Step 6: Commit** — `feat(api)!: ログインを Firebase の ID トークン交換 (/auth/firebase) に置き換える`

### Task 4: ローカル結合確認

テスト鍵で署名したトークンと、テスト JWKS を配る簡易サーバーで、DB を含めて確認する。

- [ ] **Step 1:** ローカル SurrealDB（前回と同じもの）に向けて、空きポート 3003 で CMS を起動する。テナントは `fbtest`。環境変数は前回のローカル確認と同じに加えて `FIREBASE_PROJECT_ID=test-project`、`FIREBASE_JWKS_URL=http://127.0.0.1:8765/jwks.json`。`python3 -m http.server 8765` を scratchpad で `jwks.json`（= `firebase_test_jwks.json`）を置いて起動。いずれも自分で起動した pid だけ後で止める
- [ ] **Step 2:** scratchpad の Python（PyJWT が無ければ `cargo test` の `sign` を `#[ignore]` テストで出力）で `valid_claims()` 相当のトークンを作り、`curl -X POST localhost:3003/t/fbtest/auth/firebase -d '{"id_token":"..."}'` → `{token}`。同じトークンで 2 回目 → JWT の `sub` が同じ
- [ ] **Step 3:** 得た JWT で `GET /t/fbtest/collections` → 200。改ざんトークン → 401。`/auth/signin` → 404
- [ ] **Step 4:** プロセスを止める

### Task 5: CMS web

**Files:**
- Create: `web/src/api/firebase.ts`、`web/src/pages/SignInPage.test.tsx`
- Modify: `web/src/pages/SignInPage.tsx`、`web/src/App.tsx`、`web/src/components/Layout.tsx`、`web/src/api/client.ts`、`web/package.json`
- Delete: `web/src/pages/ChangePasswordPage.tsx`、`web/src/pages/ChangePasswordPage.test.tsx`

- [ ] **Step 1:** `cd web && npm install firebase`
- [ ] **Step 2: `api/firebase.ts`**

```ts
import { initializeApp } from 'firebase/app'
import {
  GoogleAuthProvider,
  getAuth,
  inMemoryPersistence,
  setPersistence,
  signInWithPopup,
  signOut,
} from 'firebase/auth'
import { BASE_URL } from './tenant'

// Firebase の Web config は公開値。別プロジェクトを使うときは VITE_FIREBASE_* で上書きする
const app = initializeApp({
  apiKey:
    import.meta.env.VITE_FIREBASE_API_KEY ??
    'AIzaSyBWgxz2n9U1akZYh7d8-2UmniFnYGVDsW8',
  authDomain:
    import.meta.env.VITE_FIREBASE_AUTH_DOMAIN ?? 'botcast-cms.firebaseapp.com',
  projectId: import.meta.env.VITE_FIREBASE_PROJECT_ID ?? 'botcast-cms',
})

/** Google でログインし、Firebase の ID トークンを CMS の JWT に交換する */
export async function signInWithGoogle(): Promise<string> {
  const auth = getAuth(app)
  // 以後は CMS の JWT だけを使うので Firebase のセッションは残さない
  await setPersistence(auth, inMemoryPersistence)
  const credential = await signInWithPopup(auth, new GoogleAuthProvider())
  const idToken = await credential.user.getIdToken()
  await signOut(auth)

  const res = await fetch(`${BASE_URL}/auth/firebase`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ id_token: idToken }),
  })
  if (!res.ok) {
    const text = await res.text()
    throw new Error(text || `${res.status} ${res.statusText}`)
  }
  const data = (await res.json()) as { token: string }
  return data.token
}
```

- [ ] **Step 3: 失敗するテスト `SignInPage.test.tsx`**

```tsx
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { MemoryRouter } from 'react-router-dom'
import { describe, expect, it, vi } from 'vitest'
import { SignInPage } from './SignInPage'

const signIn = vi.fn()
vi.mock('../contexts/auth', () => ({ useAuth: () => ({ signIn }) }))
vi.mock('../api/firebase', () => ({
  signInWithGoogle: vi.fn().mockResolvedValue('cms-token'),
}))

describe('SignInPage', () => {
  it('Google でログインすると CMS のトークンを保存する', async () => {
    render(
      <MemoryRouter>
        <SignInPage />
      </MemoryRouter>,
    )
    await userEvent.click(screen.getByRole('button', { name: /Google/ }))
    expect(signIn).toHaveBeenCalledWith('cms-token')
  })
})
```

既存テストの import（`@testing-library/user-event` の有無など）に合わせる。無ければ `fireEvent.click` と `await vi.waitFor(...)` を使う。
`npm test -- SignInPage` → FAIL

- [ ] **Step 4: `SignInPage.tsx`** — `callAuthApi` / email / password / mode を削除し、ボタン 1 つにする。既存のカード風のスタイルは残す

```tsx
// @specre 01KQS7YE9H3RWJSHBNWPEJGYAV
import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../contexts/auth'
import { signInWithGoogle } from '../api/firebase'

export function SignInPage() {
  const { signIn } = useAuth()
  const navigate = useNavigate()
  const [error, setError] = useState<string | null>(null)
  const [loading, setLoading] = useState(false)

  async function handleClick() {
    setError(null)
    setLoading(true)
    try {
      signIn(await signInWithGoogle())
      navigate('/')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Authentication failed')
    } finally {
      setLoading(false)
    }
  }
  // return: 既存の外枠 div 2 つ + <h2>Sign In — Botcast CMS</h2>
  //   + {error && <p style={{ color: 'red' }}>{error}</p>}
  //   + <button onClick={handleClick} disabled={loading}>{loading ? 'Signing in...' : 'Sign in with Google'}</button>
}
```

- [ ] **Step 5:** `App.tsx` の `ChangePasswordPage` import と `/account/password` ルート、`Layout.tsx` の Password リンク、`client.ts` の `authApi` を削除。2 ファイルを削除
- [ ] **Step 6:** `npm test && npm run build && npm run lint` → PASS
- [ ] **Step 7: Commit** — `feat(web): サインインを Google ログインのみにし、パスワード変更画面を削除する`

### Task 6: 設定・ドキュメント・spec カード・バージョン

- [ ] **Step 1:** `deploy/compose.yaml` の api の environment に `- FIREBASE_PROJECT_ID=${FIREBASE_PROJECT_ID:-botcast-cms}`
- [ ] **Step 2:** README / ARCHITECTURE.md / CLAUDE.md の signup / signin / change-password の記述を `/auth/firebase` と `FIREBASE_PROJECT_ID` / `FIREBASE_JWKS_URL` に書き換え（`git grep -n -E "signup|signin|change-password|argon"` で漏れを確認。`middleware/tenant.rs` のパス文字列のテストはそのまま）
- [ ] **Step 3: spec カード**
  - `01KQS7YE9H5SMQB6BBNKNSVW8T`: ファイル名・`name` を「Google アカウント (Firebase Auth) でサインインできる」にし、機能概要・主要なメンバー・シナリオを新方式に書き換える（シナリオ: 初回ログインでユーザー作成 / 2 回目は同じユーザー / `aud`・`iss`・期限切れ・署名不正は 401 / 未知の `kid` は JWKS を取り直す / `FIREBASE_PROJECT_ID` 未設定は 500 / 無効なトークン・別テナントのトークンは従来どおり）。関連ファイルに `cms/crates/api/src/firebase.rs`
  - `01M4HXZFXP92HEM8FPRHJNXZW3`（管理画面でパスワードを変更できる）: カードを削除し、他カードの `[[管理画面でパスワードを変更できる]]` リンクを消す
  - `01KQS7YE9H3RWJSHBNWPEJGYAV`（フロントエンドで JWT を管理）: サインイン手段の記述を更新
  - `specre index` → `specre drift` → 触ったカードを `specre verify <id>`、`specre orphans` で迷子が無いこと
- [ ] **Step 4:** バージョンを 0.7.0 に（`cms/crates/api/Cargo.toml`、`Cargo.lock`、`web/package.json`、web の lockfile。0.6.0 のときと同じ箇所）
- [ ] **Step 5: Commit・PR** — PR タイトル `feat!: ログインを Firebase Auth (Google) のみにする`、base `develop`。CI（specre は UTC）が通ったら squash merge の確認をユーザーに取る

## botcast

### Task 7: botcast web

**Files:**
- Create: `web/src/lib/firebase.ts`
- Modify: `web/src/routes/signin.lazy.tsx`、`web/src/routes/users/$userId.lazy.tsx`、`web/package.json`
- Delete: `web/src/components/user/ChangePasswordForm.tsx`

- [ ] **Step 1:** `cd web && npm install firebase`
- [ ] **Step 2: `lib/firebase.ts`** — CMS web の `api/firebase.ts` と同じ内容。違いは `BASE_URL` の代わりに `import { CMS_URL } from "@/lib/cms_client"` で `${CMS_URL}/auth/firebase`、エラーは `body.message` を優先（既存の signin と同じ扱い）、タブインデント・ダブルクォート（biome）
- [ ] **Step 3: `signin.lazy.tsx`**

```tsx
import { Button } from "@/components/ui/button";
import { useSession } from "@/hooks/useSession";
import { signInWithGoogle } from "@/lib/firebase";
import { createLazyFileRoute, useNavigate } from "@tanstack/react-router";
import { useState } from "react";

export const Route = createLazyFileRoute("/signin")({
	component: Signin,
});

function Signin() {
	const { saveToken } = useSession();
	const navigate = useNavigate();
	const [error, setError] = useState<string | null>(null);
	const [loading, setLoading] = useState(false);

	const handleSignIn = async () => {
		setLoading(true);
		setError(null);
		try {
			saveToken(await signInWithGoogle());
			navigate({ to: "/" });
		} catch (e) {
			setError(e instanceof Error ? e.message : "サインインに失敗しました");
		} finally {
			setLoading(false);
		}
	};

	return (
		<div className="flex flex-col gap-4 max-w-sm mx-auto mt-16 p-4">
			<h2 className="text-xl font-bold">サインイン</h2>
			{error && <p className="text-red-500 text-sm">{error}</p>}
			<Button onClick={handleSignIn} disabled={loading}>
				{loading ? "サインイン中..." : "Google でサインイン"}
			</Button>
		</div>
	);
}
```

- [ ] **Step 4: `users/$userId.lazy.tsx`** — `ChangePasswordForm` と `isMe` の分岐を削除し、見出しとユーザー ID だけにする。`ChangePasswordForm.tsx` を削除
- [ ] **Step 5:** `npm run check` → PASS（biome の format が差分を出したら `npm run format` 相当で直す）
- [ ] **Step 6: Commit** — `feat(web): サインインを Google ログインのみにする`

### Task 8: 移行 SurrealQL とドキュメント

- [ ] **Step 1: `deploy/selfhost/migrations/2026-10-15-firebase-users.surql`**

```sql
-- パスワード認証の旧ユーザー (uid が無い user) が持つレコードを、Firebase でログインした新ユーザーに付け替えて旧ユーザーを消す。
-- 新ユーザーが 1 人だけいる状態で流す (botcast web で Google ログインを 1 回済ませておく)。
LET $new = (SELECT VALUE <string> id FROM user WHERE uid != NONE);
IF array::len($new) != 1 { THROW "Firebase でログインした user がちょうど 1 人である必要があります" };
LET $old = (SELECT VALUE <string> id FROM user WHERE uid = NONE);
FOR $c IN (SELECT id, name FROM collection WHERE name INSIDE ['podcasts', 'episodes', 'scripts', 'mails']) {
    UPDATE type::table(record::id($c.id)) SET owner_id = $new[0] WHERE owner_id INSIDE $old;
};
DELETE user WHERE uid = NONE;
```

前回のローカル SurrealDB（`ownertest` 等）で、旧ユーザー 1 + 新ユーザー 1 + レコードを作って流し、`owner_id` が付け替わり旧ユーザーが消えることを確認する

- [ ] **Step 2:** `CLAUDE.md` の認証の節（`/auth/signin` → Firebase の Google ログインで ID トークンを得て `POST /t/{tenantId}/auth/firebase` で CMS JWT に交換。config は `web/src/lib/firebase.ts`、`VITE_FIREBASE_*` で上書き可）、`crates/worker/src/lib.rs:15` の説明を更新
- [ ] **Step 3: Commit・PR** — ブランチ名を `feat/firebase-auth` にして push、PR タイトル `feat: ログインを Firebase Auth (Google) のみにする`。CMS の PR がマージ・リリースされてからマージする

### Task 9: リリースと本番移行（各手順でユーザーに確認）

1. ユーザーに Firebase の承認済みドメインへ cdfy-dev の CMS web / botcast web のホストを追加してもらう
2. CMS の PR をマージし 0.7.0 をリリース。cdfy-dev の CMS compose に `FIREBASE_PROJECT_ID` を反映（compose の変更は手動）。Watchtower の更新を待ち、api のログに `Auth schema initialized` を確認
3. botcast の PR をマージ、web の更新を待つ
4. CMS の SurrealDB をバックアップ（`~/backups/cms-botcast-<date>-firebase.surql`）
5. ユーザーが botcast web で Google ログイン
6. 移行 SurrealQL を流す（確認を取ってから）。`SELECT owner_id, count() FROM <各テーブル> GROUP BY owner_id` で新ユーザーに付いたこと、`SELECT count() FROM user` が 1 を確認
7. botcast web で番組の編集導線が出ること、CMS web で Google ログインできることをユーザーに確認してもらう
