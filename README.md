# Majher

**Majher** to przenośny program dla Windows do dzielenia długich audiobooków i innych plików audio na krótsze części.

## Najważniejsze funkcje

- podział według wybranej długości fragmentu,
- tryb audiobooka wykorzystujący metadane, pauzy i lokalne rozpoznawanie zapowiedzi rozdziałów,
- formaty MP3, M4A AAC, WAV, FLAC, OPUS oraz podział bez rekompresji,
- automatyczna obsługa FFmpeg i lokalnego whisper.cpp,
- przyspieszenie NVIDIA CUDA z bezpiecznym powrotem do CPU,
- interfejs dostosowany do obsługi klawiaturą i czytnikiem ekranu NVDA.

## Pobieranie

Najnowsza wersja znajduje się w sekcji [Releases](https://github.com/proszczok2002/majher/releases/latest).

Pobierz `Majher-1.6.zip`, rozpakuj archiwum i uruchom `Majher.exe`. Program nie wymaga instalatora.

> Windows może przy pierwszym uruchomieniu wyświetlić komunikat SmartScreen, ponieważ plik wykonywalny nie ma komercyjnego podpisu cyfrowego.

## Aktualizacje

Program sprawdza nowe wersje wyłącznie w GitHub Releases. Pobrany plik EXE jest weryfikowany opublikowaną sumą SHA-256 przed podmianą programu.
Pliki aktualizatora oraz konfiguracje FFmpeg i whisper.cpp są dołączane jako zasoby każdego wydania.

## Prywatność

Wszystkie pliki audio i krótkie próbki używane do wykrywania rozdziałów są przetwarzane lokalnie na komputerze. Nagrania nie są wysyłane do usług internetowych.

## Dostępność

Interfejs korzysta ze standardowych kontrolek Windows, obsługuje nawigację Tab i Shift+Tab oraz współpracuje z NVDA.

## Kod źródłowy

To repozytorium służy wyłącznie do dystrybucji gotowych wydań. Kod źródłowy programu nie jest publikowany.

