# Mermaid 다이어그램 안내

`docs/diagrams`는 `kwd` 문서에서 재사용하는 Mermaid 소스와 설정을 모아 두는 디렉터리입니다.

## 현재 포함된 파일

- `component.mmd`: `kwd`, Kaniko, 레지스트리, 호출자, `copier`의 관계
- `sequence.mmd`: `kwd`가 환경 변수를 읽고 Kaniko로 `exec`하는 실행 흐름
- `mermaid-config.json`: 저장소 공용 Mermaid 렌더링 스타일

`README.md`에는 같은 다이어그램이 Mermaid fenced block으로 포함되어 있습니다. 문서를 수정할 때는 `README.md`와 이 디렉터리의 `.mmd` 파일이 서로 같은 내용을 설명하는지 함께 확인하세요.

## 렌더링 규칙

- 수정 기준 파일은 `*.mmd`입니다.
- 렌더 산출물(`.svg`, `.png`)은 필요할 때만 생성합니다.
- 별도 브라우저 launch 설정 파일은 저장소에 추적하지 않습니다. Mermaid CLI 실행에 필요한 Chromium/Puppeteer 준비는 실행 환경에서 해결합니다.

## 예시 명령

단일 SVG 렌더링:

```bash
bunx @mermaid-js/mermaid-cli \
  -i docs/diagrams/component.mmd \
  -o docs/diagrams/component.svg \
  -c docs/diagrams/mermaid-config.json
```

단일 PNG 렌더링:

```bash
bunx @mermaid-js/mermaid-cli \
  -i docs/diagrams/sequence.mmd \
  -o docs/diagrams/sequence.png \
  -c docs/diagrams/mermaid-config.json \
  --scale 2
```

## 변경 후 확인

- 다이어그램 설명이 `README.md`의 프로젝트 계약 및 동작 설명과 모순되지 않는지 검토합니다.
- 특히 `kwd`의 태그 처리, `--destination` 생성, Kaniko `exec` 흐름과 일치해야 합니다.
- 렌더 산출물을 생성했다면 사람이 읽을 수 있는지 직접 열어 확인합니다.
- 문서 변경 후에는 `git diff --check`로 공백 오류를 확인합니다.
