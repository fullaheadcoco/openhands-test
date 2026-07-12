# OpenHands로 Rust TUI Todo 앱 만들기 - 실전 후기

> **OpenHands**: AI 기반 소프트웨어 개발 자동화 플랫폼  
> **기간**: 2026-07-11 (약 2시간)  
> **Repo**: [fullaheadcoco/openhands-test](https://github.com/fullaheadcoco/openhands-test)

---

## 1. OpenHands란?

자연어로 지시하면 AI가 **직접 코드를 읽고, 쓰고, 빌드하고, GitHub에 올리는** 개발 자동화 도구입니다.

| 기존 AI 코딩 도구 | OpenHands |
|---|---|
| 코드 조각 제안 | 파일 전체를 읽고 수정 |
| 채팅창에서만 동작 | 터미널, 브라우저, 파일 시스템 직접 조작 |
| 사람이 복붙 | AI가 직접 실행하고 결과 확인 |
| 한 번에 한 가지 | 여러 도구 조합해서 끝까지 해결 |

---

## 2. 전체 과정 요약

```
"자동화 만들어줘"
  → GitHub 토큰 등록
  → "rust tui todo app 만들고 이슈, 마일스톤 만들어서 진행"
  → 앱 전체 구현 + GitHub 프로젝트 관리
  → 정렬 기능 추가
  → 멀티에이전트 병렬 작업 (테스트 + CI/CD + 문서화)
  → CI 오류 디버깅 & 수정
```

### GitHub 마일스톤 진행

| # | 마일스톤 | 이슈 | 상태 |
|---|---------|------|:--:|
| 1 | MVP - 기본 기능 | #1~#5 CRUD, 우선순위, 검색 | ✅ |
| 2 | Persistence | #6~#7 JSON 저장/로딩 | ✅ |
| 3 | Enhancements | #8~#10 편집, 우선순위, 검색 | ✅ |
| 4 | v0.2 - 정렬 | #11~#13 정렬 모드, 타임스탬프 | ✅ |
| 5 | v0.3 - 품질 | #14~#16 테스트, CI/CD, 문서 | ✅ |

**16개 이슈, 5개 마일스톤, 모두 자동 생성 및 종료**

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
| 삭제 | `d` | 선택 항목 삭제 |
| 완료 토글 | `Space` | ✓ / ☐ |
| 우선순위 | `p` | 🔴HIGH → 🟡MED → 🟢LOW |
| 검색 | `/` | 키워드 필터 |
| 정렬 | `s` | PRI → NEW → OLD → A-Z |
| 종료 | `q` | 자동 저장 후 종료 |

**기술 스택**: Rust · Ratatui · Crossterm · Serde(JSON)  
**코드**: `src/main.rs` ~500줄, `cargo build` 경고 0개

---

## 4. 멀티에이전트 데모

OpenHands의 핵심 기능 — **3개 AI가 동시에 다른 작업을 병렬 처리**:

```
POST /api/conversations  ─→  🧪 Agent 1: 유닛 테스트 작성
POST /api/conversations  ─→  🔧 Agent 2: GitHub Actions CI/CD
POST /api/conversations  ─→  📖 Agent 3: 문서화 (README + Architecture)
                                    ↓
                              병렬 실행 (각자 다른 파일)
                                    ↓
                    ✅ Tests (4/4 pass)  ✅ CI/CD  ✅ Docs
```

**핵심**: 각 에이전트가 **서로 다른 파일**만 건드리도록 설계 → 충돌 없음

| 파일 | 작성자 |
|------|--------|
| `src/main.rs` (테스트 모듈) | Agent 1 |
| `.github/workflows/rust.yml` | Agent 2 |
| `docs/README.md`, `docs/ARCHITECTURE.md` | Agent 3 |

**결과**: 5분 만에 3가지 작업 동시 완료, PR 머지 시 충돌 zero

---

## 5. CI/CD 파이프라인

멀티에이전트가 직접 만든 GitHub Actions:

```yaml
name: Rust
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:   # cargo build --release
  test:    # cargo test
  fmt:     # cargo fmt --check
```

**실제 실행 이력**:

| 커밋 | build | test | fmt | 링크 |
|------|:--:|:--:|:--:|------|
| `043f27f` | ✅ | ✅ | ✅ | [성공](https://github.com/fullaheadcoco/openhands-test/actions/runs/29173518031) |
| `2c7b26a` | ✅ | ✅ | ❌ | [fmt 실패](https://github.com/fullaheadcoco/openhands-test/actions/runs/29173425816) |

> **교훈**: 첫 시도에서 `fmt`가 실패했지만, OpenHands가 자동으로 원인 파악 → `cargo fmt` 적용 → 재푸시 → 전부 통과. CI가 있어서 바로 잡을 수 있었음.

---

## 6. OpenHands의 실제 작업 패턴

### 일반적인 흐름

```
사용자: "기능 추가해줘"
  ↓
1. GitHub 이슈/마일스톤 생성 (프로젝트 관리)
  ↓
2. 코드 탐색 → 이해
  ↓
3. 구현 (파일 편집, 빌드, 테스트)
  ↓
4. Git 커밋 & 푸시 (Closes #N 으로 이슈 자동 종료)
  ↓
5. CI 결과 확인 → 실패 시 디버깅 → 수정
```

### 사용 가능한 도구들

| 도구 | 용도 |
|------|------|
| `file_editor` | 파일 읽기/쓰기/편집 |
| `terminal` | 쉘 명령어 실행, 빌드, 테스트 |
| `browser` | 웹페이지 탐색, API 문서 확인 |
| `task_tracker` | 작업 계획 및 진행 관리 |
| `github` | Issues, PRs, Milestones, Actions |

---

## 7. 한계 & 배운 점

| 상황 | 배운 점 |
|------|---------|
| CI `fmt` 실패 | 멀티에이전트 간 컨벤션 공유 필요 (`cargo fmt`은 작업 후 필수) |
| 한글 PDF 생성 | 폰트 이슈 — 시스템 폰트가 없으면 fallback 필요 |
| 같은 파일 동시 편집 | 에이전트가 같은 파일을 건드리면 충돌 → 파일 단위로 작업 분리 |
| 로컬 = 웹훅 불가 | 크론 폴링으로 대체해야 함 |

---

## 8. 시작하는 방법

### 기본 사용법

```
# 대화 시작
"Rust로 TODO 앱 만들어줘"

# GitHub 연동
"이 레포에 이슈 만들어서 진행해줘"
"https://github.com/myorg/myrepo"

# 멀티에이전트
"테스트랑 CI/CD랑 문서화를 동시에 진행해줘"
```

### Automation (자동화)

```bash
# 5분마다 새 이슈 확인 → 자동 분석 → 코멘트
curl -X POST ".../api/automation/v1/preset/prompt" \
  -d '{
    "name": "Issue Triage",
    "prompt": "Analyze new issues and suggest labels",
    "trigger": {"type": "cron", "schedule": "*/5 * * * *"}
  }'
```

---

## 9. 정리

| 항목 | 결과 |
|------|------|
| 코드 | Rust TUI 앱 ~500줄, 빌드 성공 |
| 테스트 | 4/4 통과 |
| CI/CD | GitHub Actions 자동화 완료 |
| 문서 | README + Architecture 문서화 |
| 프로젝트 관리 | 이슈 16개, 마일스톤 5개 |
| 멀티에이전트 | 3개 병렬 작업 성공 |

**핵심 요약**:  
OpenHands는 그냥 "코드 추천"이 아니라, **PM → 개발 → 테스트 → CI/CD → 문서화**까지 개발 전 과정을 AI가 직접 수행하는 도구입니다. 자연어로 지시하면 끝까지 해냅니다.

---

> *이 문서는 OpenHands AI 에이전트가 실제 작업 과정을 바탕으로 생성했습니다.*  
> *Repo: https://github.com/fullaheadcoco/openhands-test*