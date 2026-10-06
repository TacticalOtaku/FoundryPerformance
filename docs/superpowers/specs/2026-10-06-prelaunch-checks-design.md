# Проверки до запуска: ускорение и статус хостинга (дизайн)

Дата: 2026-10-06 · Статус: утверждён в обсуждении

Дополнение к `2026-10-06-gpu-acceleration-check-design.md`. Пользователь хочет видеть
доступность GPU-ускорения и статус сервера **до** запуска игры, а не после.

## 1. Статус серверов на хостинге Sqyre

### Что выяснено (2026-10-06, анонимными запросами)
- Слот хранит адрес страницы хостинга: `https://www.sqyre.app/games/<slug>/`. Без входа в
  аккаунт Sqyre она отвечает «This game may not exist yet, or may not be public».
- Foundry живёт на `https://<slug>.games.sqyre.app/` (адрес назвал пользователь).
  - `/api/status` закрыт прокси Sqyre: 404 `{"success":false,"message":"Not found"}`;
  - `/game` → 302 на `/join`, заголовок `X-Powered-By: Express` — это сам Foundry, мир запущен;
  - `/join` перехватывает nginx Sqyre (302 на страницу игры);
  - несуществующий slug → 502 Bad Gateway (предположительно так же отвечает спящий сервер).
- **Баг:** проверка `FoundryUrl` в `report_telemetry` требует `probe(url).foundry`, а на Sqyre
  `/api/status` закрыт — отчёт отклоняется, страница игры не становится доверенной, и с v0.2.2
  лаунчер отклоняет все отчёты агента с Sqyre (сессии, замеры, профиль; и проверка GPU).

### Решение
1. `probe::hosted_foundry(url: &Url) -> Option<Url>`: для `www.sqyre.app` / `sqyre.app` и пути
   ровно `/games/<slug>/`, где `slug` из `[a-z0-9-]+`, — `https://<slug>.games.sqyre.app/`.
   Иначе `None`.
2. `probe::probe(input)` проверяет `hosted_foundry(base)`, если он есть, иначе сам адрес.
   Фронт по-прежнему шлёт `gameUrl ?? url` — для Sqyre-слота подстановка происходит в Rust.
3. Запасная проба, если `/api/status` не ответил JSON Foundry:
   - ответ `/api/status` с кодом 5xx → сервер выключен (`ProbeResult::default()`);
   - иначе `GET <base>game` **без следования редиректам**, разбор `parse_game_redirect(status, location)`:
     - 3xx и путь `Location` оканчивается на `/join` → `foundry: true, active: true`;
     - 3xx и путь оканчивается на `/setup`, `/auth` или `/license` → `foundry: true, active: false`;
     - 5xx → `ProbeResult::default()` (выключен);
     - прочее → `reachable: true`, не Foundry.
   - `version`, `world`, `system`, `users` в этом случае `None`.
4. `telemetry::session_origins(server)` добавляет происхождение `hosted_foundry` для адреса
   слота — страница игры Sqyre доверенная с момента запуска, без зависимости от пробы.
5. Опрос раз в минуту не меняется.

## 2. Ускорение до запуска — WebGL лаунчера

Выбран вариант «WebGL в окне лаунчера» (не скрытое окно-проба в окружении игры):
мгновенно и бесплатно, но это профиль лаунчера со стоковыми флагами — ответ на вопрос
«доступен ли GPU для WebView2», а не «как будет у игры с этим движком».

- Фронт при загрузке: `launcherRenderer()` создаёт невидимый `canvas`, контекст `webgl2`
  (иначе `webgl`), читает `UNMASKED_RENDERER_WEBGL`, освобождает контекст через
  `WEBGL_lose_context`.
- Команда `classify_renderer(renderer: String) -> Result<Verdict, String>` — `gpu::classify` с
  адаптерами DXGI; строка длиннее 512 символов → `Err`. Доступна только окну лаунчера.
  Ничего не сохраняет: проверка свежая на каждом старте.
- Паспорт, `accelView(settings, current, launcher)`:
  1. есть актуальная проверка из игры и вердикт не `unknown` → она (`source: "game"`), как раньше;
  2. иначе есть вердикт лаунчера, не `unknown` → он (`source: "launcher"`); строка API — движок
     из настроек; `wrongGpu` лаунчера показывается как `on`: лаунчер рисует без
     `--force-high-performance-gpu`, на ноутбуке он законно на встройке, а игра попросит дискретную;
  3. иначе «НЕ ПРОВЕРЕНО».
- Подсказка различает источник: ключи `accel.tip.launcher.on`, `accel.tip.launcher.software`.
- Программный рендер, найденный лаунчером, движок **не переключает** — только красная лампочка
  и совет обновить драйвер. Автооткат остаётся за проверкой в игре.
- Превью: `?launcher=on|software|wrongGpu|none` в `mock.ts`.

## 3. Тесты
- Rust `probe`: `hosted_foundry` — Sqyre с и без `www`, с `/` и без на конце, чужой хост, лишние
  сегменты, slug с запрещёнными символами; `parse_game_redirect` — `/join`, `/setup`, `/auth`,
  абсолютный `Location`, 200, 404, 502.
- Rust `telemetry`: `session_origins` для Sqyre-слота содержит `https://<slug>.games.sqyre.app`.
- Rust `commands`: длинная строка в `classify_renderer` отклоняется (через чистую функцию).
- Вживую (ignored-тест): `probe("https://www.sqyre.app/games/aldarionv210-3c11b9b6/")` — `foundry: true`
  при работающем сервере.
- TS: `launcherRenderer` на поддельном canvas; `accelView` — приоритет игры над лаунчером,
  `wrongGpu` лаунчера → `on`, API из настроек для источника «лаунчер».
- Превью: состояния `?launcher=` при отсутствии проверки из игры (`?accel=none`).
- Смоук портативной сборки: на машине с GPU паспорт сразу показывает «ВКЛ».
