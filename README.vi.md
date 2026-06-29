# DeepSeek

> Coding agent trong terminal cho mọi model — ưu tiên model mở.

Một TUI và CLI viết bằng Rust, 25 provider. DeepSeek, OpenRouter, Hugging Face,
DeepInfra và vLLM/SGLang/Ollama chạy cục bộ là các đường first-class, và
DeepSeek nói chuyện native với Anthropic Claude và OpenAI khi đó là thứ bạn
đang có. Công cụ qua cổng phê duyệt, sandbox cấp hệ điều hành, và rollback
bằng `/restore` cho mọi lượt.

<<<<<<< HEAD
![deepseek screenshot](assets/screenshot.png)
=======
[English README](README.md) · [简体中文 README](README.zh-CN.md) · [日本語 README](README.ja-JP.md) · [한국어 README](README.ko-KR.md) · [deepseek.net](https://deepseek.net/) · [Hướng dẫn cài đặt](docs/INSTALL.md) · [Danh mục provider](docs/PROVIDERS.md) · [Changelog](CHANGELOG.md)
>>>>>>> 0fd9515922b72c5120c813e785f489d0b261fd9e

[![CI](https://github.com/coohu/deepseek-tui/actions/workflows/ci.yml/badge.svg)](https://github.com/coohu/deepseek-tui/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/deepseek-cli?label=crates.io)](https://crates.io/crates/deepseek-cli)
[![npm](https://img.shields.io/npm/v/deepseek?label=npm)](https://www.npmjs.com/package/deepseek)
[![DeepWiki project index](https://img.shields.io/badge/DeepWiki-project-blue)](https://deepwiki.com/coohu/deepseek-tui)

<<<<<<< HEAD
Phần lớn coding agent bắt đầu bằng sức mạnh: nhiều công cụ hơn, context dài hơn, tự động hóa nhiều hơn. DeepSeek bắt đầu bằng trách nhiệm.

Trước khi một agent sửa repo, nó cần một địa chỉ: terminal này, người dùng này, branch này, session này. Đó là lớp ego. Không phải khoe mẽ, mà là tính liên tục. Không phải mặt nạ personality, mà là nơi trách nhiệm bám vào.

Sau đó nó cần luật. Workspace thật là một chồng xung đột: ý định hiện tại của người dùng, hướng dẫn trong repo, output từ shell, memory cũ, handoff cũ, chính sách an toàn và thay đổi đang dang dở có thể va vào nhau trong cùng một lượt. Constitution của DeepSeek xếp thứ tự cho các nguồn đó: yêu cầu hiện tại cao hơn ngữ cảnh cũ; bằng chứng trực tiếp cao hơn phỏng đoán; kiểm chứng cao hơn sự tự tin; personality chỉ điều chỉnh giọng nói, không quyết định hành động.

Sản phẩm thật là lớp sắp thứ tự quanh model: ai đang hành động, luật nào thắng, chứng cứ nào tồn tại, và người hoặc agent tiếp theo có thể tiếp tục ra sao.

## DeepSeek cung cấp gì

- TUI chạy cục bộ trong terminal.
- Công cụ có schema cho file, Shell, Git, Web, MCP, RLM và sub-agent.
- Cổng phê duyệt, sandbox, snapshot side-git và rollback bằng `/restore`.
- Phản hồi diagnostics từ language server sau khi chỉnh sửa.
- Sub-agent chạy song song, session bền, fork, relay handoff và Runtime API.
- DeepSeek V4 là đường chính, cùng các provider rõ ràng như OpenRouter, Xiaomi MiMo, NVIDIA NIM, Arcee, SiliconFlow, Fireworks, Novita, SGLang/vLLM tự host, Ollama và các bề mặt Hugging Face khi chúng được hoàn thiện.

DeepSeek là first-class, nhưng không phải giới hạn duy nhất. Provider, model, base URL và credentials là các lựa chọn tách biệt.
=======
![DeepSeek chạy trong terminal](assets/screenshot.png)
>>>>>>> 0fd9515922b72c5120c813e785f489d0b261fd9e

## Cài đặt

```bash
<<<<<<< HEAD
cargo install deepseek-cli --locked
cargo install deepseek-tui --locked
deepseek --version
deepseek --model auto
=======
npm install -g deepseek
deepseek --version   # 0.8.65
>>>>>>> 0fd9515922b72c5120c813e785f489d0b261fd9e
```

Wrapper npm (Node 18+) tải binary đã xác minh SHA-256 từ GitHub Releases và
cài các lệnh `deepseek`, `codew` và `deepseek-tui`. Muốn tự build từ source?
Dùng cargo (Rust 1.88+):

```bash
<<<<<<< HEAD
# GitHub Releases có archive theo nền tảng:
# https://github.com/coohu/deepseek-tui/releases

# Nếu GitHub không ổn định, dùng CNB mirror:
cargo install --git https://cnb.cool/deepseek.net/deepseek --tag v0.8.54 deepseek-cli --locked --force
cargo install --git https://cnb.cool/deepseek.net/deepseek --tag v0.8.54 deepseek-tui --locked --force

# Homebrew legacy trong lúc formula vẫn dùng tên deepseek-tui
brew tap coohu/deepseek-tui
brew install deepseek-tui
```

Wrapper npm `deepseek` cho v0.8.54 được hoãn có chủ ý trong lúc đường phát hành release asset được gia cố. Với bản này, hãy dùng Cargo, GitHub Releases hoặc CNB.

Docker, tải trực tiếp, mirror Trung Quốc, Windows/Scoop, Nix, checksum và troubleshooting nằm trong [docs/INSTALL.md](docs/INSTALL.md).
=======
cargo install deepseek-cli --locked
cargo install deepseek-tui --locked
```

> **Người dùng Linux:** cài đặt các gói build trước:
> `sudo apt-get install -y build-essential pkg-config libdbus-1-dev`.
> Xem [INSTALL.md](docs/INSTALL.md#4-install-via-cargo-any-tier-1-rust-target).

Mọi đường cài đặt khác:

```bash
# Docker
docker pull ghcr.io/hmbown/deepseek:latest

# Nix
nix run github:coohu/deepseek-tui

# Windows
scoop install deepseek        # hoặc trình cài NSIS từ GitHub Releases

# CNB mirror cho người dùng khó truy cập GitHub ổn định
cargo install --git https://cnb.cool/deepseek.net/deepseek --tag v0.8.65 deepseek-cli --locked --force
cargo install --git https://cnb.cool/deepseek.net/deepseek --tag v0.8.65 deepseek-tui --locked --force

# Homebrew legacy trong lúc formula đang được đổi tên
brew tap Hmbown/deepseek-tui
brew install deepseek-tui
```

Archive dựng sẵn cho mọi nền tảng — bao gồm cả Linux riscv64 — được đính kèm
trong [GitHub Releases](https://github.com/coohu/deepseek-tui/releases).
Checksum, mirror Trung Quốc, chi tiết riêng cho Windows và troubleshooting nằm
trong [docs/INSTALL.md](docs/INSTALL.md).
>>>>>>> 0fd9515922b72c5120c813e785f489d0b261fd9e

## Lần chạy đầu tiên

```bash
deepseek auth set --provider deepseek
deepseek auth status
deepseek doctor
deepseek
```

Mọi provider đều cùng một dạng lệnh một dòng: `--provider openrouter`,
`--provider moonshot`, hoặc trỏ `vllm`, `sglang`, `ollama` vào runtime
localhost của riêng bạn mà không cần key nào cả. Có key Claude? Chạy
`deepseek auth set --provider anthropic` — hoặc chỉ cần export
`ANTHROPIC_API_KEY` — và adapter Messages native sẽ lo phần còn lại.

Key được lưu trong `~/.deepseek/config.toml`; cấu hình cũ trong
`~/.deepseek/` vẫn được đọc để giữ tương thích.

Các lệnh hữu ích trong session:

- `/provider` và `/model` đổi đường định tuyến và model ngay giữa session.
- `/restore` quay lui một lượt trước đó từ snapshot side-git.
- `/skills` nạp các workflow tái sử dụng từ `~/.deepseek/skills/`.
- `/config` chỉnh cài đặt runtime; `/statusline` hiển thị route hiện tại,
  chi phí và trạng thái session.
- `! cargo test -p deepseek-tui` chạy bất kỳ lệnh shell nào qua đường
  approval và sandbox bình thường.

Chế độ headless, cho script và CI:

```bash
deepseek exec --allowed-tools read_file,exec_shell --max-turns 10 "fix the failing test"
```

## DeepSeek cung cấp gì

Một harness agent thuần terminal — TUI + CLI, 16 crate Rust — nơi các rào an
toàn là cơ chế runtime, không phải lời dặn mà model phải tự nhớ:

- **Công cụ qua cổng phê duyệt với sandbox cấp HĐH.** Công cụ file, shell,
  git, web, MCP và sub-agent chạy sau cổng phê duyệt tường minh và các backend
  sandbox (bwrap, Landlock, Seatbelt, seccomp).
- **Rollback đáng tin cậy.** Snapshot side-git và `/restore`, giữ bên ngoài
  `.git` của repo — hoàn tác một lượt không bao giờ chạm vào lịch sử của bạn.
- **Hooks v2**. Hook `tool_call_before` trả về quyết định JSON
  `allow`/`deny`/`ask` với quy tắc deny thắng, matcher dạng glob, và
  `.deepseek/hooks.toml` riêng cho từng dự án.
- **Sub-agent chạy song song với định tuyến theo provider**. Điều tra và triển
  khai song song, với các tier model lớn/rẻ được phân giải theo từng provider —
  không hardcode model id.
- **Session bền.** Fork, relay handoff, và prompt cache lưu trên đĩa dùng
  chung giữa các session, ổn định từng byte khi chuyển qua lại giữa chế độ
  Plan/Agent/YOLO. Lượt chạy sống sót qua sleep hệ thống: máy ngủ giữa stream,
  thức dậy, request được âm thầm gửi lại thay vì làm hỏng lượt.
- **Chế độ headless.** `deepseek exec` với `--allowed-tools`,
  `--disallowed-tools` (deny thắng), `--max-turns` và `--append-system-prompt`
  cho script và CI.
- **Nhúng được ở mọi nơi.** Runtime API HTTP/SSE và ACP, extension VS Code
  (Phase 0), và cầu nối Telegram/Feishu (cầu nối Weixin đang thử nghiệm).
- **Độ hoàn thiện để dùng hằng ngày.** Vừa là MCP client *vừa* là MCP server,
  skill tái sử dụng, bản địa hóa 7 ngôn ngữ, và speech/TTS qua Xiaomi MiMo.

### Mọi model, ưu tiên model mở

Hai mươi lăm provider đi qua cùng một harness, cùng một constitution, cùng
một bộ công cụ:

- **Model mở, dạng hosted:** `deepseek` (đứng đầu trong nhóm ngang hàng),
  `openrouter`, `huggingface` (Inference Providers), `moonshot` (Kimi),
  `volcengine` (Ark), `nvidia-nim`, `together`, `fireworks`, `novita`,
  `siliconflow` / `siliconflow-CN`, `arcee`, `xiaomi-mimo`, `atlascloud`,
  `deepinfra`, `wanjie-ark`, cộng thêm một đường `openai`-compatible tổng quát cho bất kỳ
  gateway nào.
- **Model mở, tự host:** `vllm`, `sglang` và `ollama` trỏ vào endpoint
  localhost của riêng bạn — không cần key.
- **Provider đóng, hỗ trợ native:** `anthropic` qua adapter `/v1/messages`
  chuyên dụng với adaptive thinking, breakpoint prompt-cache và phát lại
  signed-thinking — không phải shim giả giọng OpenAI — và `openai-codex`, tái
  sử dụng phiên đăng nhập ChatGPT/Codex CLI sẵn có.

Định tuyến không chỉ là đổi base URL: mức effort của `/reasoning` được dịch
sang phương ngữ wire của từng provider, tier sub-agent phân giải theo
provider, và phần facts về model trong system prompt được template theo từng
model thay vì hardcode. Đổi giữa session bằng `/provider` và
`/model`. Danh mục đầy đủ — credentials, base URL, ranh giới năng lực — nằm
trong [docs/PROVIDERS.md](docs/PROVIDERS.md).

Fanout của sub-agent ưu tiên cấu hình. Đặt mặc định trong `[subagents]`, rồi
thêm `[subagents.providers.deepseek]`, `[subagents.providers.glm]`,
`[subagents.providers.openrouter]` hoặc profile provider khác để khớp API bạn
đang dùng. Direct DeepSeek có thể mở rộng; route subscription hoặc dễ bị rate
limit có thể giữ ở 3–5 agent song song mà không đổi prompt hay code. Xem
[docs/SUBAGENTS.md](docs/SUBAGENTS.md#concurrency-cap).

Các nhãn phiên bản ở trên đánh dấu những gì đã hạ cánh trong ba bản phát hành
gần nhất (0.8.56 → 0.8.58). Chi tiết đầy đủ trong [CHANGELOG.md](CHANGELOG.md).

## Ý tưởng chính — mission idea được đưa vào phiên bản này

Phần lớn coding agent bắt đầu bằng việc thêm sức mạnh: nhiều công cụ hơn,
context dài hơn, tự chủ nhiều hơn. DeepSeek bắt đầu bằng việc gán trách
nhiệm.

(Đây là mission thiết kế đang được đưa vào phiên bản này; memory, cost,
và remote orchestration vẫn đang lặp lại — xem v0.9.0 Track bên dưới.)

Một agent sửa repo của bạn cần có một địa chỉ — terminal này, người dùng này,
branch này, session này. Không phải một persona; một địa chỉ để truy hồi. Khi
có gì đó hỏng, "model làm đấy" không phải là câu trả lời. "Instance này, trong
session này, sau lần phê duyệt này" mới là câu trả lời.

Sau đó nó cần luật. Một phiên làm việc thật là một chồng xung đột: yêu cầu
hiện tại của bạn, chỉ dẫn trong repo, output shell vừa chạy, memory cũ, và
bản handoff của agent trước đó cùng tranh nhau trong một lượt. **Constitution
của DeepSeek** cố định thứ tự quyền lực:

1. **Ý định người dùng là tối thượng.** Yêu cầu hiện tại của bạn đứng trên
   hướng dẫn repo đã cũ, memory, handoff trước đó và các lớp personality.
2. **Luật của repo phải tường minh.** Thêm `.deepseek/constitution.json` để
   khai báo quyền lực bền vững của dự án: các bất biến cần bảo vệ, chính sách
   branch, quy tắc kiểm chứng.
3. **Bằng chứng đứng trên lời kể.** Output của công cụ thắng một phỏng đoán
   tự tin. `cargo test` thất bại được báo cáo đúng là `cargo test` thất bại,
   không bao giờ bị tóm tắt thành lạc quan. Kiểm chứng là một phần của nhiệm
   vụ, không phải phần vĩ thanh.
4. **Memory xếp cuối.** Hữu ích, nhưng không bao giờ có thẩm quyền.

Phần chính sách quan trọng được thực thi bằng code, không phải bằng prompt:
cổng phê duyệt, sandbox, snapshot, rollback và schema công cụ là các cơ chế
runtime mà model không thể nói khéo để lách qua.

Và không phần nào của bộ luật đó nằm trong model — vì thế model mới thay
được. Harness mang constitution; model cung cấp khả năng suy luận. DeepSeek
và thế giới open-weight là công dân hạng nhất, một chiếc máy trong LAN của
bạn chạy vLLM hay Ollama là một peer đầy đủ, và khi thứ bạn có là key Claude
hay OpenAI, DeepSeek cũng nói các API đó một cách native.

Đó chính là sản phẩm: không phải một model lớn hơn, mà một harness nghiêm
khắc hơn quanh bất kỳ model nào bạn chọn. Đổi model; luật vẫn đứng vững.

## Tài liệu chi tiết

<<<<<<< HEAD
README chỉ giữ ý tưởng và đường đi nhanh nhất. Chi tiết nằm trong docs và [deepseek.net](https://deepseek.net/):
=======
README giữ phần ý tưởng và con đường đầu tiên. Chi tiết nằm trong docs và
trên [deepseek.net](https://deepseek.net/):
>>>>>>> 0fd9515922b72c5120c813e785f489d0b261fd9e

- [User guide](docs/GUIDE.md) — giờ đầu tiên với DeepSeek.
- [Install guide](docs/INSTALL.md) — mọi đường cài đặt và troubleshooting.
- [Configuration](docs/CONFIGURATION.md) — file cấu hình, constitution của
  repo và cài đặt provider.
- [Provider registry](docs/PROVIDERS.md) — đường model, credentials, base URL
  và ranh giới năng lực.
- [Sub-agents](docs/SUBAGENTS.md) — vai trò, vòng đời, hợp đồng output và
  hành vi phục hồi.
- [MCP](docs/MCP.md) — kết nối tool server bên ngoài và chạy DeepSeek như
  một MCP server.
- [Runtime API](docs/RUNTIME_API.md) — hợp đồng tích hợp HTTP/SSE, ACP,
  mobile và GUI/editor.
- [Model Lab](docs/MODEL_LAB.md) — roadmap khám phá và đánh giá model mở.
- [Architecture](docs/ARCHITECTURE.md) — bố cục crate, luồng runtime, hệ
  thống công cụ, điểm mở rộng và mô hình bảo mật.

## Track v0.9.0

v0.9.0 là làn tích hợp hiện tại. Những việc đang tụ về đó:

- bề mặt relay và handoff mạnh hơn giữa các session và agent;
- transcript gọn gàng hơn cho các chuỗi công cụ dày đặc;
- runtime API cho VS Code và các client GUI;
- điều phối workflow branch/leaf với WhaleFlow.

Chi tiết theo từng bản phát hành nằm trong [CHANGELOG.md](CHANGELOG.md).

## Lời cảm ơn

<<<<<<< HEAD
- **[DeepSeek](https://github.com/deepseek-ai)** — Xin chân thành cảm ơn sự hỗ trợ và các mô hình AI mạnh mẽ giúp tiếp sức cho mọi tương tác trong dự án. 感谢 DeepSeek 提供模型与支持，让每一次交互成为可能。
- **[DataWhale](https://github.com/datawhalechina)** 🐋 — Xin cảm ơn sự hỗ trợ nhiệt tình và đã chào đón chúng tôi gia nhập gia đình lớn "Whale Brother". 感谢 DataWhale 的支持，并欢迎 chúng tôi gia nhập “鲸兄弟”大家庭。
- **[OpenWarp](https://github.com/zerx-lab/warp)** — Cảm ơn vì đã ưu tiên hỗ trợ deepseek và hợp tác để mang lại trải nghiệm agent terminal tốt hơn.
- **[Open Design](https://github.com/nexu-io/open-design)** — Cảm ơn vì sự hỗ trợ và hợp tác xung quanh quy trình làm việc chú trọng thiết kế của agent.
=======
- **[DeepSeek](https://github.com/deepseek-ai)** — Xin cảm ơn các model và sự
  hỗ trợ đã tiếp sức cho mọi lượt tương tác.
  感谢 DeepSeek 提供模型与支持，让每一次交互成为可能。
- **[DataWhale](https://github.com/datawhalechina)** 🐋 — Xin cảm ơn sự hỗ trợ
  nhiệt tình và đã chào đón chúng tôi vào đại gia đình "Whale Brother".
  感谢 DataWhale 的支持，并欢迎我们加入“鲸兄弟”大家庭。
- **[OpenWarp](https://github.com/zerx-lab/warp)** — Cảm ơn vì đã ưu tiên hỗ
  trợ deepseek và hợp tác để mang lại trải nghiệm agent terminal tốt hơn.
- **[Open Design](https://github.com/nexu-io/open-design)** — Cảm ơn vì sự hỗ
  trợ và hợp tác xung quanh quy trình làm việc chú trọng thiết kế của agent.
>>>>>>> 0fd9515922b72c5120c813e785f489d0b261fd9e

Dự án này được phát hành với sự giúp sức của một cộng đồng đóng góp ngày càng
lớn. Nguyên tắc của maintainer rất đơn giản: báo cáo lỗi và PR là công việc
thực sự của dự án, kể cả khi bản vá cuối cùng phải được thu hẹp, hoãn lại,
hoặc harvest vào một nhánh của maintainer.

<<<<<<< HEAD
Các đóng góp đã được merge hoặc được harvest trong v0.8.48: **[@cy2311](https://github.com/cy2311)**, **[@LING71671](https://github.com/LING71671)**, **[@axobase001](https://github.com/axobase001)**, **[@dzyuan](https://github.com/dzyuan)**, **[@mvanhorn](https://github.com/mvanhorn)**, **[@malsony](https://github.com/malsony)**, **[@gaord](https://github.com/gaord)**, **[@yuanchenglu](https://github.com/yuanchenglu)**, **[@idling11](https://github.com/idling11)**, **[@h3c-hexin](https://github.com/h3c-hexin)**, **[@AdityaVG13](https://github.com/AdityaVG13)**, **[@Sskift](https://github.com/Sskift)**, **[@cyq1017](https://github.com/cyq1017)**, **[@HUQIANTAO](https://github.com/HUQIANTAO)**, **[@New2Niu](https://github.com/New2Niu)**, **[@AiurArtanis](https://github.com/AiurArtanis)**, **[@Lee-take](https://github.com/Lee-take)**, **[@nightt5879](https://github.com/nightt5879)**, **[@AresNing](https://github.com/AresNing)**, **[@AccMoment](https://github.com/AccMoment)**, **[@reidliu41](https://github.com/reidliu41)**, **[@aboimpinto](https://github.com/aboimpinto)**, **[@zhuangbiaowei](https://github.com/zhuangbiaowei)**, **[@donglovejava](https://github.com/donglovejava)**, **[@hongqitai](https://github.com/hongqitai)**, **[@zlh124](https://github.com/zlh124)**, **[@encyc](https://github.com/encyc)**, **[@Implementist](https://github.com/Implementist)**, **[@lihuan215](https://github.com/lihuan215)**, **[@LeoAlex0](https://github.com/LeoAlex0)**, **[@jimmyzhuu](https://github.com/jimmyzhuu)**, **[@rockyzhang](https://github.com/rockyzhang)**, **[@mo-vic](https://github.com/mo-vic)**, **[@hufanexplore](https://github.com/hufanexplore)**, **[@hoclaptrinh33](https://github.com/hoclaptrinh33)** và **[@BryonGo](https://github.com/BryonGo)**.

Xin cảm ơn các báo cáo, bước tái hiện lỗi và xác minh từ **[@buko](https://github.com/buko)**, **[@yyyCode](https://github.com/yyyCode)**, **[@gaslebinh-glitch](https://github.com/gaslebinh-glitch)**, **[@Dr3259](https://github.com/Dr3259)**, **[@lpeng1711694086-lang](https://github.com/lpeng1711694086-lang)**, **[@VerrPower](https://github.com/VerrPower)**, **[@yan-zay](https://github.com/yan-zay)**, **[@jretz](https://github.com/jretz)**, **[@Neo-millunnium](https://github.com/Neo-millunnium)**, **[@caeserchen](https://github.com/caeserchen)**, **[@T-Phuong-Nguyen](https://github.com/T-Phuong-Nguyen)**, **[@zhyuzhyu](https://github.com/zhyuzhyu)**, **[@0gl20shk0sbt36](https://github.com/0gl20shk0sbt36)**, **[@hatakes](https://github.com/hatakes)**, **[@goodvecn-dev](https://github.com/goodvecn-dev)**, **[@bevis-wong](https://github.com/bevis-wong)**, **[@PurplePulse](https://github.com/PurplePulse)** và **[@nbiish](https://github.com/nbiish)** đã giúp định hình v0.8.48.

- **[merchloubna70-dot](https://github.com/merchloubna70-dot)** — Đóng góp 28 PR bao gồm tính năng mới, sửa lỗi và dựng sẵn extension cho VS Code (#645–#681)
- **[WyxBUPT-22](https://github.com/WyxBUPT-22)** — Xây dựng trình kết xuất Markdown hỗ trợ bảng biểu, chữ đậm/nghiêng và đường kẻ ngang (#579)
- **[loongmiaow-pixel](https://github.com/loongmiaow-pixel)** — Tài liệu cài đặt cho Windows và Trung Quốc (#578)
- **[20bytes](https://github.com/20bytes)** — Cải tiến tài liệu tính năng tự ghi nhớ và giao diện trợ giúp (#569)
- **[staryxchen](https://github.com/staryxchen)** — Kiểm tra độ tương thích của thư viện glibc trước khi chạy (#556)
- **[Vishnu1837](https://github.com/Vishnu1837)** — Tối ưu hóa tính tương thích glibc và tự phục hồi trạng thái terminal khi nhận tín hiệu SIGINT/SIGTERM (#565, #1586)
- **[shentoumengxin](https://github.com/shentoumengxin)** — Kiểm tra hợp lệ ranh giới thư mục làm việc `cwd` của Shell (#524)
- **[toi500](https://github.com/toi500)** — Báo cáo và sửa lỗi dán văn bản trên hệ điều hành Windows
- **[xsstomy](https://github.com/xsstomy)** — Báo cáo lỗi vẽ lại màn hình khi khởi động terminal
- **Melody0709** — Báo cáo lỗi kích hoạt phím Enter với tiền tố lệnh gạch chéo
- **[lloydzhou](https://github.com/lloydzhou)** và **[jeoor](https://github.com/jeoor)** — Báo cáo lỗi chi phí nén dữ liệu; lloydzhou cũng đóng góp ngữ cảnh môi trường xác định (#813, #922) và ổn định bộ nhớ đệm KV prefix-cache (#1080)
- **[Agent-Skill-007](https://github.com/Agent-Skill-007)** — Tinh chỉnh diễn đạt rõ ràng cho file giới thiệu README (#685)
- **[woyxiang](https://github.com/woyxiang)** — Tài liệu hướng dẫn cài đặt qua Scoop trên Windows (#696)
- **[wangfeng](mailto:wangfengcsu@qq.com)** — Cập nhật thông tin giá cả và chương trình khuyến mãi (#692)
- **[zichen0116](https://github.com/zichen0116)** — Xây dựng tài liệu quy tắc ứng xử cộng đồng CODE_OF_CONDUCT.md (#686)
- **[dfwqdyl-ui](https://github.com/dfwqdyl-ui)** — Báo cáo tính tương thích chữ hoa/thường của ID mô hình (#729)
- **[Oliver-ZPLiu](https://github.com/Oliver-ZPLiu)** — Báo cáo lỗi trạng thái `working...` bị kẹt, cơ chế dự phòng khay nhớ tạm (clipboard) trên Windows, sửa lỗi phiên kết nối HTTP dạng MCP Streamable, và tự động hóa brew tap (#738, #850, #1643, #1631)
- **[reidliu41](https://github.com/reidliu41)** — Ý tưởng gợi ý tiếp tục phiên, lưu trữ độ tin cậy workspace, hỗ trợ nhà cung cấp Ollama, hoàn thiện stream khối suy nghĩ, tăng cường cache cho CI, xử lý wrap dòng stream, và hoàn thành tính năng autocomplete cho DeepSeek (#863, #870, #921, #1078, #1603, #1628, #1601)
- **[xieshutao](https://github.com/xieshutao)** — Cơ chế dự phòng skill dạng Markdown thuần (#869)
- **[GK012](https://github.com/GK012)** — Cơ chế dự phòng lệnh `--version` của wrapper npm (#885)
- **[y0sif](https://github.com/y0sif)** — Xử lý đánh thức vòng lặp agent cha sau khi các sub-agent con hoàn thành tác vụ (#901)
- **[mac119](https://github.com/mac119)** và **[leo119](https://github.com/leo119)** — Viết tài liệu hướng dẫn cho lệnh `deepseek update` (#838, #917)
- **[dumbjack](https://github.com/dumbjack)** / **浩淼的mac** — Tăng cường bảo mật chống mã độc qua lệnh shell byte rỗng (#706, #918)
- **macworkers** — Cải tiến xác nhận rẽ nhánh (fork) kèm mã phiên làm việc mới (#600, #919)
- **zero** và **[zerx-lab](https://github.com/zerx-lab)** — Cấu hình điều kiện nhận thông báo và làm phong phú nội dung thông báo qua OSC 9 (#820, #920)
- **[chnjames](https://github.com/chnjames)** — Gợi ý hoàn thành @mentions từ cache, cải tiến phục hồi file cấu hình lỗi, và hiển thị chuẩn UTF-8 cho Shell trên Windows (#849, #927, #982, #1018)
- **[angziii](https://github.com/angziii)** — Bảo mật cấu hình, dọn dẹp tài nguyên bất đồng bộ, tăng cường bảo mật Docker và vá lỗi an toàn thực thi lệnh (#822, #824, #827, #831, #833, #835, #837)
- **[elowen53](https://github.com/elowen53)** — Giải mã UTF-8 và bổ sung các ca kiểm thử xác định (#825, #840)
- **[wdw8276](https://github.com/wdw8276)** — Bổ sung lệnh `/rename` để đổi tên tiêu đề phiên làm việc tùy chỉnh (#836)
- **[banqii](https://github.com/banqii)** — Hỗ trợ đường dẫn tìm kiếm skill dạng `.cursor/skills` (#817)
- **[junskyeed](https://github.com/junskyeed)** — Tính toán động giá trị `max_tokens` cho các yêu cầu API (#826)
- **Hafeez Pizofreude** — Triển khai cơ chế chống tấn công SSRF trong công cụ `fetch_url` và biểu đồ lịch sử Star History.
- **Unic (YuniqueUnic)** — Xây dựng giao diện cấu hình tự động dựa trên schema (cả TUI và web).
- **Jason** — Tăng cường bảo mật an toàn mạng chống tấn công giả mạo yêu cầu từ phía máy chủ (SSRF).
- **[axobase001](https://github.com/axobase001)** — Dọn dẹp snapshot mồ côi, bổ sung bộ bảo vệ khi cài npm, sửa lỗi đo lường phiên làm việc, xóa cache phạm vi mô hình, hỗ trợ các liên kết tượng trưng (symlinks) cho skill, hướng dẫn cơ chế thoát lỗi cài đặt npm mirror, và duy trì cấu hình proxy cho các tác vụ con (#975, #1032, #1047, #1049, #1052, #1019, #1051, #1056, #1608)
- **[MengZ-super](https://github.com/MengZ-super)** — Xây dựng nền tảng cho lệnh `/theme` và giải nén dữ liệu nén dạng gzip/brotli cho kết nối SSE (#1057, #1061)
- **[DI-HUO-MING-YI](https://github.com/DI-HUO-MING-YI)** — Vá lỗi bảo mật sandbox chỉ đọc trong chế độ Plan (#1077)
- **[bevis-wong](https://github.com/bevis-wong)** — Cung cấp ca tái hiện chính xác lỗi tự động gửi tin khi dán văn bản kèm ký tự xuống dòng (#1073)
- **[Duducoco](https://github.com/Duducoco)** và **[AlphaGogoo](https://github.com/AlphaGogoo)** — Xây dựng thanh menu gạch chéo cho skill và sửa lỗi bao phủ lệnh `/skills` (#1068, #1083)
- **[ArronAI007](https://github.com/ArronAI007)** — Sửa lỗi hiển thị tài nguyên artifact khi thay đổi kích thước cửa sổ trên macOS Terminal.app và ConHost (#993)
- **[THINKER-ONLY](https://github.com/THINKER-ONLY)** — Duy trì mã mô hình tùy chỉnh cho OpenRouter và endpoint riêng (#1066)
- **[Jefsky](https://github.com/Jefsky)** — Báo cáo sửa lỗi địa chỉ endpoint chính thức của DeepSeek (#1079, #1084)
- **[wlon](https://github.com/wlon)** — Chẩn đoán và ưu tiên lựa chọn khóa xác thực cho nhà cung cấp NVIDIA NIM (#1081)
- **[Horace Liu](https://github.com/liuhq)** — Đóng gói hỗ trợ Nix package và viết tài liệu hướng dẫn cài đặt (#1173)
- **[jieshu666](https://github.com/jieshu666)** — Giảm thiểu hiện tượng nhấp nháy màn hình khi vẽ lại giao diện TUI (#1563)
- **[gordonlu](https://github.com/gordonlu)** — Sửa lỗi nhận dạng phím Enter / mã nhập CSI-u trên Windows (#1612)
- **[mdrkrg](https://github.com/mdrkrg)** — Vá lỗi sập ứng dụng trong lần chạy đầu tiên khi thiếu khóa API (#1598)
- **[Aitensa](https://github.com/Aitensa)** — Xử lý tự động xuống dòng CJK cho các khối diff và kết quả đầu ra trang giấy (#1622)
- **[qiyan233](https://github.com/qiyan233)** — Đảm bảo tương thích với các bí danh cũ của nhà cung cấp DeepSeek Trung Quốc (#1645)
- **[zlh124](https://github.com/zlh124)** — Báo cáo khởi động không đầu WSL2 và sửa lỗi khay nhớ tạm (#1772, #1773)
- **[aboimpinto](https://github.com/aboimpinto)** — Sửa lỗi ghi nhật ký màn hình phụ trên Windows, hoàn thiện phím Home/End tại bộ soạn thảo và theo dõi log runtime (#1774, #1776, #1748, #1749, #1782, #1783)
- **[LeoLin990405](https://github.com/LeoLin990405)** — Bổ sung cơ chế truyền thẳng mô hình qua provider, phát lại luồng suy nghĩ, tối ưu lượt chạy chỉ suy nghĩ, và sửa lỗi trích dẫn trên Windows (#1740, #1743, #1742, #1744)
- **[nightt5879](https://github.com/nightt5879)** — Khắc phục lỗi khôi phục giao diện nhắc nhở khi bấm phím Ctrl+C (#1764)
- **[donglovejava](https://github.com/donglovejava)** — Hợp nhất kéo thả dán tệp `@file`, vá lỗi sập chữ CJK, thu thập phản hồi người dùng, định tuyến RLM, và thử lại khi `edit_file` bị kẹt (#2154–#2168)
- **[encyc](https://github.com/encyc)** — Hiển thị chi tiết số lượng token tiêu thụ ở chân trang và lệnh `/status` (#2152)
- **[saieswar237](https://github.com/saieswar237)** — Bổ sung tài liệu hướng dẫn về quy trình review code (#2178)
- **[sximelon](https://github.com/sximelon)** — Chặn sự kiện tự gửi tin khi dán văn bản và tách phân hệ quản lý phím bấm (#2174, #2042)
- **[nanookclaw](https://github.com/nanookclaw)** — Bổ sung hiển thị nhà cung cấp tìm kiếm trong kết quả của lệnh doctor (#2135)
- **[Sskift](https://github.com/Sskift)** — Ngăn chặn việc ghi đè biến môi trường mặc định trên CLI (#2119)
- **[xin1104](https://github.com/xin1104)** — Tạo brew formula cài binary deepseek độc lập (#2105)
- **[mrluanma](https://github.com/mrluanma)** — Bổ sung nhà cung cấp dịch vụ tìm kiếm Metaso (#2059)
- **[Lellansin](https://github.com/Lellansin)** — Bỏ qua việc gộp cấu hình tại thư mục home người dùng (#2055)
- **[zhuangbiaowei](https://github.com/zhuangbiaowei)** — Cập nhật các kênh phát hành chính thức của sản phẩm (#2145)
=======
Danh sách ghi công đầy đủ theo từng người đóng góp — và luôn được cập nhật —
nằm trong [mục Thanks của README tiếng Anh](README.md#thanks), hồ sơ ghi nhận
chính thức của dự án.
>>>>>>> 0fd9515922b72c5120c813e785f489d0b261fd9e

---

## Đóng góp cho dự án

<<<<<<< HEAD
Xem tài liệu hướng dẫn đóng góp tại [CONTRIBUTING.md](CONTRIBUTING.md). Chúng tôi luôn hoan nghênh các yêu cầu kéo Pull Requests — vui lòng xem danh sách các [vấn đề mở (open issues)](https://github.com/coohu/deepseek-tui/issues) để bắt đầu đóng góp những phần việc đầu tiên.
=======
Xem [CONTRIBUTING.md](CONTRIBUTING.md). Hoan nghênh các Pull Request — hãy xem
[danh sách issue đang mở](https://github.com/coohu/deepseek-tui/issues) để tìm
những đóng góp đầu tiên phù hợp.
>>>>>>> 0fd9515922b72c5120c813e785f489d0b261fd9e

DeepSeek nhận được rất nhiều báo cáo và PR chất lượng. Lập trường của
maintainer là giữ cánh cửa đó luôn mở trong khi vẫn bảo vệ chất lượng phát
hành:

- Issue nên dễ đọc với con người và có thể hành động được. Tự động hóa khâu
  tiếp nhận chỉ mang tính tư vấn, trừ khi maintainer chủ động bật chế độ
  cưỡng chế.
- PR được review từ code, test, issue liên quan và hành vi runtime, không chỉ
  từ tiêu đề.
- Nếu một PR quá rộng để merge trực tiếp, maintainer có thể harvest phần an
  toàn vào một nhánh hẹp hơn, sau đó ghi công tác giả và giải thích phần nào
  đã được đưa vào.
- Trailer co-author nên dùng danh tính GitHub noreply có thể ánh xạ từ
  `.github/AUTHOR_MAP`; người báo cáo và người viết bước tái hiện lỗi nên
  được cảm ơn trong changelog, release notes và bình luận khi đóng issue.
- Người đóng góp thường xuyên có thể được thêm vào
  `.github/APPROVED_CONTRIBUTORS` để các cổng dry-run không cản đường họ.

Ủng hộ dự án: [Buy me a coffee](https://www.buymeacoffee.com/hmbown).

> [!NOTE]
> *Dự án này không trực thuộc DeepSeek Inc.*

## Giấy phép

[MIT](LICENSE)

## Star History

<<<<<<< HEAD
[![Biểu đồ lịch sử sao](https://api.star-history.com/chart?repos=coohu/deepseek-tui&type=date&legend=top-left)](https://www.star-history.com/?repos=coohu%2FCodeWhale&type=date&logscale=&legend=top-left)
=======
[![Biểu đồ Star History](https://api.star-history.com/chart?repos=coohu/deepseek-tui&type=date&legend=top-left)](https://www.star-history.com/?repos=Hmbown%2FCodeWhale&type=date&logscale=&legend=top-left)
>>>>>>> 0fd9515922b72c5120c813e785f489d0b261fd9e
