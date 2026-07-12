# OpenHands로 Rust TUI Todo 앱 만들기 — 실전 후기

> **OpenHands**: AI 기반 소프트웨어 개발 자동화 플랫폼  
> **기간**: 2026-07-11 (약 2시간)  
> **Repo**: [fullaheadcoco/openhands-test](https://github.com/fullaheadcoco/openhands-test)

---

## 1. OpenHands란?

### 기존 AI 코딩 도구와의 근본적 차이

기존 도구들(GitHub Copilot, Cursor, ChatGPT 등)은 **"코드를 추천"** 하는 수준에서 멈춥니다.
OpenHands는 **"개발자처럼 행동"** 합니다.

| | 기존 AI 도구 | OpenHands |
|---|---|---|
| 코드 | 조각 추천 | 파일 전체 읽고 수정 |
| 실행 | 사람이 복붙 | AI가 `cargo build` 직접 실행 |
| 디버깅 | 사람이 함 | 빌드 실패 → 원인 분석 → 수정 |
| Git | 사람이 commit | AI가 commit & push |
| 프로젝트 관리 | 별도 도구 | GitHub 이슈/마일스톤 자동 생성 |
| CI/CD | 사람이 설정 | AI가 workflow yml 작성 |
| 작업 단위 | 한 번에 한 가지 | 여러 도구 조합해 끝까지 |

### 핵심: "끝까지 해낸다"

```
사용자: "rust tui todo app 만들어줘"
  ↓
1. GitHub 마일스톤 & 이슈 생성 (계획 수립)
  ↓
2. Cargo init, 의존성 추가 (프로젝트 초기화)
  ↓
3. main.rs 전체 구현 (코드 작성)
  ↓
4. cargo build → 오류 수정 → 빌드 성공 (검증)
  ↓
5. git commit & push → 이슈 자동 종료 (배포)
```

이 전체 사이클에 **사람의 개입이 없습니다.** 자연어 지시 한 줄이면 끝입니다.

### OpenHands가 다룰 수 있는 것들

```
┌─ 대화창 ───────────────────────────────────────┐
│  "버그 고쳐줘"                                  │
│  "기능 추가해줘"                                │
│  "리팩토링 해줘"                                │
│  "테스트 작성해줘"                              │
└────────────────────────────────────────────────┘
                    ↓
┌─ AI Agent ────────────────────────────────────┐
│                                                │
│  🔧 Terminal    cargo build, git push, curl    │
│  📝 File Editor 파일 읽기/쓰기/수정             │
│  🌐 Browser     API 문서, 웹 조사               │
│  📋 TaskTracker 작업 계획 & 진행 추적            │
│  🐙 GitHub      Issues, PRs, Milestones        │
│  ⚡ Automation  크론, 웹훅, 멀티에이전트         │
│                                                │
└────────────────────────────────────────────────┘
```

---

## 2. 전체 과정

### 타임라인

```
"자동화 만들어줘"
  → GitHub 토큰 등록, Automation 백엔드 확인
  → "rust tui todo app 만들고 이슈, 마일스톤 만들어서 진행"
  → 앱 전체 구현 + GitHub 프로젝트 관리 (마일스톤 3개, 이슈 10개)
  → 정렬 기능 추가 (마일스톤 4, 이슈 3개)
  → 멀티에이전트 병렬 작업 (마일스톤 5, 이슈 3개)
  → CI fmt 실패 → 사용자 보고 → 수정
  → CI Failure Auto-Fix Automation 등록
```

### GitHub 마일스톤

| # | 마일스톤 | 이슈 | 상태 |
|---|---------|------|:--:|
| 1 | MVP — 기본 기능 | #1~#5 CRUD, 우선순위, 검색 | ✅ |
| 2 | Persistence | #6~#7 JSON 저장/로딩 | ✅ |
| 3 | Enhancements | #8~#10 편집, 우선순위, 검색 | ✅ |
| 4 | v0.2 — 정렬 | #11~#13 정렬 모드, 타임스탬프 | ✅ |
| 5 | v0.3 — 품질 | #14~#16 테스트, CI/CD, 문서 | ✅ |

**16개 이슈, 5개 마일스톤 — 생성부터 종료까지 모두 AI가 처리**

---

## 3. 앱 스펙

```
📋 Rust TODO | PRI | n:add e:edit d:del s:sort ␣:done p:pri /:find q:quit
┌─ Todos (2/5) ────────────────────────────────────┐
│  HIGH  ✓ ~~json 저장 구현~~                        │
│  MED   ✓ ~~기본 UI 렌더링~~                         │
│  LOW   ☐ 에러 핸들링 추가                            │
│  HIGH  ☐ 키보드 단축키 도움말                         │
│  MED   ☐ 다크 모드 지원                              │
└───────────────────────────────────────────────────┘
┌─ Status ─────────────────────────────────────────┐
│  Selected: "에러 핸들링 추가" | LOW | Pending        │
└───────────────────────────────────────────────────┘
```

| 기능 | 키 | 설명 |
|------|:--:|------|
| 추가 | `n` | 모달 입력 → Enter 저장 |
| 편집 | `e` | 인라인 수정 |
| 삭제 | `d` / `Delete` | 선택 항목 삭제 |
| 완료 토글 | `Space` | ✓ / ☐ |
| 우선순위 | `p` | 🔴HIGH → 🟡MED → 🟢LOW |
| 검색 | `/` | 키워드 필터링 |
| 정렬 | `s` | PRI → NEW → OLD → A-Z |
| 종료 | `q` / `Esc` | todos.json 자동 저장 |

**기술 스택**: Rust · Ratatui · Crossterm · Serde(JSON)  
**코드**: `src/main.rs` ~500줄, `cargo build` 경고 0개  
**파일 구조**:

```
rust-todo/
├── src/main.rs              # 전체 앱
├── Cargo.toml               # 의존성
├── .github/workflows/rust.yml  # CI/CD (build/test/fmt)
├── docs/
│   ├── README.md            # 사용자 가이드
│   └── ARCHITECTURE.md      # 설계 문서
├── agents_orchestrator.py   # 멀티에이전트 스크립트
└── OPENHANDS_INTRO.md       # 이 문서
```

---

## 4. 멀티에이전트

OpenHands의 핵심 기능 — **3개 AI가 동시에 다른 작업을 병렬 처리**:

```
POST /api/conversations  ─→  🧪 Agent 1: src/main.rs 테스트 모듈
POST /api/conversations  ─→  🔧 Agent 2: .github/workflows/rust.yml
POST /api/conversations  ─→  📖 Agent 3: docs/*.md
                                    ↓
                    동시 실행 (서로 다른 파일만 건드림)
                                    ↓
              ✅ 4/4 tests pass  ✅ CI/CD  ✅ 문서화 완료
```

**설계 원칙**: 각 에이전트가 **완전히 다른 파일/디렉토리**만 작업 → 충돌 zero

| 파일 | 에이전트 |
|------|----------|
| `src/main.rs` (+67줄 테스트) | 🧪 Tests |
| `.github/workflows/rust.yml` | 🔧 CI/CD |
| `docs/README.md`, `docs/ARCHITECTURE.md` | 📖 Docs |

**실제 오케스트레이션 코드**: [`agents_orchestrator.py`](https://github.com/fullaheadcoco/openhands-test/blob/main/agents_orchestrator.py)

---

## 5. CI/CD 파이프라인

멀티에이전트(🔧 CI/CD)가 직접 작성한 GitHub Actions:

```yaml
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:   # cargo build --release
  test:    # cargo test (4/4 pass)
  fmt:     # cargo fmt --check
```

**실제 실행 이력**:

| 커밋 | build | test | fmt | 링크 |
|------|:--:|:--:|:--:|------|
| `9fce42f` | ✅ | ✅ | ✅ | 통과 |
| `043f27f` | ✅ | ✅ | ✅ | [통과](https://github.com/fullaheadcoco/openhands-test/actions/runs/29173518031) |
| `2c7b26a` | ✅ | ✅ | ❌ | [fmt 실패](https://github.com/fullaheadcoco/openhands-test/actions/runs/29173425816) |

> `2c7b26a`에서 `fmt`이 실패했지만, `cargo fmt` 적용 후 `043f27f`에서 전부 통과.

---

## 6. Automation — CI 실패 자동 감지

워크플로우 실패를 **사람이 말하기 전에 AI가 먼저 알아채도록** Automation을 등록했습니다.

```
⏰ 5분마다 실행 (cron)
  → GitHub API: 최신 workflow run 확인
  → 모든 Job 통과? → ALL_CLEAR (종료)
  → 실패 발견? → 로그 분석 → 코드 수정 → commit & push
```

```bash
# 등록된 Automation
curl -X POST ".../api/automation/v1/preset/prompt" \
  -d '{
    "name": "CI Failure Auto-Fix",
    "prompt": "Check latest workflow runs. If failed, analyze & fix.",
    "trigger": {"type": "cron", "schedule": "*/5 * * * *"}
  }'
```

**등록 결과**: `6b2ae776` — Enabled, 5분 주기, Asia/Seoul 시간대

> 💡 로컬 환경이라 webhook 대신 cron polling 사용. Cloud 환경이면 GitHub webhook으로 **즉시** 반응 가능.

---

## 7. OpenHands의 실제 작업 패턴

### 일반적인 개발 흐름

```
사용자: "기능 추가해줘"
  ↓
1. GitHub 이슈/마일스톤 생성        ← 프로젝트 관리
2. 코드 탐색 → 구조 파악             ← 컨텍스트 이해
3. 구현 (파일 편집)                  ← 코드 작성
4. cargo build / cargo test         ← 검증
5. git commit & push (Closes #N)    ← 배포 + 이슈 종료
6. CI 결과 확인 → 실패 시 디버깅     ← 품질 보증
```

### Automation 패턴

```
⏰ Cron:    "매일 아침 보고서 생성"
🔔 Webhook: "새 이슈 열리면 자동 분류"
🤖 Multi:   "3개 에이전트로 병렬 작업"
🩺 Monitor: "CI 실패 감지 → 자동 수정"
```

---

## 8. 한계 & 배운 점

| 상황 | 배운 점 | 해결책 |
|------|---------|--------|
| CI `fmt` 실패 | 멀티에이전트 간 컨벤션 불일치 | 작업 후 `cargo fmt` 강제 |
| 사용자가 먼저 CI 실패 발견 | 완전 자동화되려면 추가 설정 필요 | CI Failure Auto-Fix 등록 |
| 한글 PDF 생성 | 시스템 폰트 부족 | weasyprint + 한글 폰트, 또는 브라우저 기반 |
| 같은 파일 동시 편집 | 에이전트끼리 충돌 | 파일/디렉토리 단위로 작업 분리 |
| 로컬 = webhook 불가 | 외부에서 이벤트 수신 불가 | cron polling으로 대체 |

---

## 9. 팀에 도입한다면

### 어떤 작업에 적합한가

| ✅ 잘하는 것 | ❌ 안 맞는 것 |
|---|---|
| 기능 개발 (CRUD, UI, API) | 실시간 협업이 필요한 논의 |
| 버그 수정 & 디버깅 | 보안/컴플라이언스 승인 |
| 테스트 코드 작성 | 레거시 마이그레이션 (복잡한 경우) |
| 문서화, README, API docs | |
| CI/CD 파이프라인 설정 | |
| 반복적인 코드 리팩토링 | |
| 프로젝트 관리 (이슈, 마일스톤) | |

### 추천 워크플로우

```
1. AGENTS.md 작성 (프로젝트 규칙, 코딩 컨벤션)
2. 작은 이슈부터 OpenHands에 맡기기
3. PR로 결과 리뷰 (사람이 최종 승인)
4. 성공 패턴 쌓이면 Automation 등록
5. CI + Auto-Fix로 무인 운전
```

---

## 10. 정리

| 항목 | 결과 |
|------|------|
| 코드 | Rust TUI 앱 ~500줄, 빌드 성공 |
| 테스트 | 4/4 통과 (`cargo test`) |
| CI/CD | GitHub Actions (build/test/fmt) |
| 문서 | README + Architecture + 팀 소개 문서 |
| 프로젝트 관리 | 이슈 16개, 마일스톤 5개 |
| 멀티에이전트 | 3개 병렬 작업 성공 |
| Automation | CI Failure Auto-Fix 등록 완료 |

**핵심**: OpenHands는 "코드 추천 도구"가 아니라, **PM → 개발 → 테스트 → CI/CD → 문서화 → 모니터링**까지 개발 전 과정을 AI가 직접 수행하는 플랫폼입니다. 자연어로 지시하면 끝까지 해냅니다.

---

> *이 문서는 OpenHands AI 에이전트가 실제 작업을 수행하며 생성했습니다.*  
> *Repo: [github.com/fullaheadcoco/openhands-test](https://github.com/fullaheadcoco/openhands-test)*