# Audio-Backend-Integration Workflow
---
description: Implementierung des CPAL-Audio-Backends und UI-Integration
---

## Ziel
Implementiere das verpflichtende Audio-Backend (CPAL) für Windows, Linux und optional macOS, integriere die Audio-Input-Device-Auswahl in die UI, verbinde den Audio-Stream mit der Media-Pipeline und implementiere Latenz‑Kompensation.

## Schritte
1. **Branch erstellen**
   ```
   git checkout -b feature/audio-backend-integration
   ```
2. **Cargo.toml anpassen** – `audio` Feature in `mapmap-core` und `mapmap` aktivieren.
3. **Backend‑Abstraktion** – Datei `crates/mapmap-core/src/audio/backend.rs` hinzufügen mit `AudioBackend` Trait und CPAL‑Implementierung (`CpalBackend`).
4. **Platform‑spezifische Implementierung** – `#[cfg(target_os = "windows")]` etc. für WASAPI, ALSA/PulseAudio, CoreAudio.
5. **UI‑Erweiterung** – In `crates/mapmap-ui/src/dashboard.rs` Dropdown für Audio‑Input‑Device, Auswahl speichert in Config.
6. **Media‑Pipeline‑Verknüpfung** – `mapmap-media/src/pipeline.rs` um Audio‑Stream in FFT‑Analyse einspeisen.
7. **Latenz‑Kompensation** – Buffer‑Größe konfigurierbar (Standard 1024 Samples) in `audio/backend.rs`.
8. **Tests hinzufügen** – Mock‑Backend für CI, Unit‑Tests für `AudioBackend` Trait.
9. **CI‑Anpassung** – `.github/workflows/ci.yml` um `--features audio` zu bauen und Tests auszuführen.
10. **Commit & Push**
    ```
    git add .
    git commit -m "feat: integrate CPAL audio backend and UI selection"
    git push origin feature/audio-backend-integration
    ```
11. **Pull‑Request erstellen** – Titel: "Feature: Audio‑Backend‑Integration", Ziel‑Branch `main`.
12. **PR‑Checks prüfen** – Falls Fehler, lokal reproduzieren, Code anpassen, erneut pushen.
13. **Merge** – Sobald alle Checks grün, PR mergen.

## Hinweis
Alle Schritte sollten in der lokalen Entwicklungsumgebung ausgeführt werden. Bei CI‑Fehlern bitte die Fehlermeldungen hier teilen, damit wir gezielt Fixes vornehmen können.
