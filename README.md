# Majher

Majher to przenośny program dla Windows, który dzieli długie audiobooki i inne nagrania na krótsze pliki. Możesz ustawić maksymalną długość części albo włączyć tryb audiobooka, który korzysta z rozdziałów zapisanych w pliku, pauz i lokalnego rozpoznawania zapowiedzi rozdziałów.

Program działa lokalnie. Nagrania i rozpoznany tekst nie są wysyłane do usługi internetowej.

## Co potrafi

- Dzieli jeden lub wiele plików audio, zachowując format wejściowy albo zapisując MP3, M4A AAC, WAV, FLAC lub OPUS.
- W trybie audiobooka najpierw wykorzystuje rozdziały z metadanych. Gdy ich nie ma, sprawdza pauzy i krótkie próbki lokalnym modelem Whisper.
- Korzysta z FFmpeg i FFprobe dostępnych w systemie lub sam przygotowuje ich lokalną kopię.
- Pozwala wybrać opcjonalne przyspieszenie NVIDIA CUDA; gdy jest niedostępne, może pracować na CPU.
- Ma polski, angielski, niemiecki i rosyjski interfejs. Język oraz motyw systemowy, jasny lub ciemny można zmieniać w oknie programu.
- Używa standardowych kontrolek Windows, klawiatury Tab/Shift+Tab oraz nazw i opisów dostępnych dla czytników ekranu, w tym NVDA.

## Pobieranie i uruchomienie

Pobierz [najnowsze wydanie](https://github.com/proszczok2002/majher/releases/latest), rozpakuj `Majher-1.9.zip` do wybranego folderu i uruchom `Majher.exe`. Instalator nie jest potrzebny. Przy pierwszym uruchomieniu Majher zapyta o język. Później możesz go zmienić w głównym oknie.

Program zapisuje ustawienia w `Majher.ini` obok pliku EXE. Umieść go więc w folderze, do którego masz prawo zapisu, na przykład w swoim folderze użytkownika. Nie uruchamiaj EXE bezpośrednio z archiwum ZIP.

> Windows może ostrzec przez SmartScreen, ponieważ EXE nie ma komercyjnego podpisu cyfrowego.

## Minimalne wymagania

- Windows 10 lub 11 i procesor x86-64 (64-bitowy).
- Wolne miejsce na pliki wynikowe i ewentualnie pobrane narzędzia. Model do rozpoznawania rozdziałów ma około 190 MB; wariant CUDA potrzebuje więcej miejsca.
- Dostęp do internetu przy pierwszym pobieraniu brakujących składników i podczas sprawdzania aktualizacji.

Karta NVIDIA, zainstalowany FFmpeg i konto GitHub **nie są wymagane** do samego uruchomienia programu. Po przygotowaniu składników dzielenie plików odbywa się lokalnie.

## Co program pobiera, gdy czegoś brakuje

1. Majher sprawdza, czy `ffmpeg.exe` i `ffprobe.exe` są dostępne w systemie.
2. Następnie sprawdza lokalne kopie obok programu.
3. Jeśli nie ma kompletnej pary narzędzi, pobiera paczkę Windows Essentials FFmpeg, weryfikuje sumę SHA-256 opublikowaną przez dostawcę i zapisuje potrzebne pliki w `tools\ffmpeg` obok EXE. Nie instaluje FFmpeg w całym systemie.
4. Jeśli korzystasz z rozpoznawania mówionych zapowiedzi rozdziałów, program pyta przed pierwszym pobraniem modelu i silnika whisper.cpp. Pliki zapisuje w `tools\chapter-detector`. Możesz wybrać CPU albo, przy zgodnej karcie NVIDIA, CUDA.

Jeśli plik audiobooka ma już rozdziały zapisane w metadanych, program wykorzystuje je bez rozpoznawania mowy. Do samego cięcia potrzebny jest FFmpeg.

## Aktualizacje

Majher sprawdza nowe wersje w [GitHub Releases](https://github.com/proszczok2002/majher/releases). Przed podmianą programu weryfikuje pobrany EXE sumą SHA-256. Aktualizator nie korzysta z prywatnego serwera autora.

## Wsparcie

Jeśli chcesz wesprzeć rozwój programu: [Revolut](https://revolut.me/r_rosiejka) lub [PayPal](https://paypal.me/proszczok). Przyciski w aplikacji mają neutralne opisy dla czytników ekranu.

## Kod źródłowy i współpraca

Kod Majhera jest publiczny w tym repozytorium: `src/main.rs`, `Cargo.toml`, `Cargo.lock` oraz `.cargo/config.toml`. Możesz przejrzeć implementację, zrobić fork i zaproponować zmianę przez pull request. Propozycje błędów i funkcji można zgłaszać w zakładce [Issues](https://github.com/proszczok2002/majher/issues). Zasady współpracy są w [CONTRIBUTING.md](CONTRIBUTING.md).

### Budowanie na Windows

Potrzebujesz Windows 10/11 x64, stabilnego Rust z `rustup` (target `x86_64-pc-windows-msvc`) oraz narzędzi kompilacji C++ z Visual Studio Build Tools. Program nie ma zewnętrznych zależności Cargo; do samego zbudowania nie trzeba instalować FFmpeg ani Whisper.

```powershell
cargo test --locked
cargo build --release --locked
```

Gotowy plik pojawi się jako `target\release\majher_bby_proszczok.exe`. Można go skopiować jako `Majher.exe` do folderu z prawem zapisu. Pliki `version.txt`, `changelog.txt`, `chapter-detector.txt` i `ffmpeg-url.txt` są publikowane wraz z gotowym wydaniem na GitHub Releases, a nie są wymagane do kompilacji. Program pobiera konfigurację i brakujące narzędzia w trakcie działania.

Nie dodawaj do pull requestów własnych plików `Majher.ini`, modeli, pobranych narzędzi, archiwów, plików serwerowych ani gotowych EXE. Aktualne wydania i sumy kontrolne znajdują się w [Releases](https://github.com/proszczok2002/majher/releases).

Kod źródłowy Majhera jest udostępniany na licencji [MIT](LICENSE). Możesz go używać, modyfikować i rozpowszechniać, także we własnym forku, zachowując informację o prawach autorskich i tekst licencji. Pull requesty są mile widziane i podlegają przeglądowi przed połączeniem z `main`. Licencja Majhera nie zmienia licencji narzędzi zewnętrznych, które program może pobrać osobno (FFmpeg i whisper.cpp).
