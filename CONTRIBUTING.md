# Współpraca przy Majherze

Dziękuję za zainteresowanie projektem. Błędy i pomysły zgłaszaj w [Issues](https://github.com/proszczok2002/majher/issues). Przy zmianie kodu:

1. Zrób fork repozytorium i utwórz własną gałąź.
2. Zmień źródła, zachowując działanie klawiatury i standardowych kontrolek dostępnych dla czytników ekranu.
3. Uruchom `cargo test --locked` oraz `cargo build --release --locked` na Windows x64 z toolchainem MSVC. Jeżeli zmiana dotyczy interfejsu, sprawdź też ręcznie Tab/Shift+Tab i czytnik ekranu; opisz, co udało się sprawdzić.
4. Otwórz pull request do gałęzi `main`. Opisz problem, rozwiązanie i wynik testów. Dołącz zrzut ekranu tylko wtedy, gdy pomaga ocenić zmianę wyglądu.

Nie umieszczaj w zmianach haseł, tokenów, lokalnych ustawień, danych użytkowników, modeli, narzędzi pobranych przez aplikację, gotowych EXE ani plików z prywatnego serwera. Pull request nie publikuje automatycznie nowego wydania — robi to opiekun projektu po sprawdzeniu zmian. Przesyłaj tylko kod, który wolno Ci udostępnić na licencji MIT projektu.

---

# Contributing

Bug reports and feature ideas are welcome in [Issues](https://github.com/proszczok2002/majher/issues). Fork the repository, work on a branch, run `cargo test --locked` and `cargo build --release --locked` on Windows x64/MSVC, then open a pull request against `main`. Describe your changes and test results. For UI changes, check keyboard and screen-reader behavior. Do not commit credentials, user data, downloaded tools/models, binaries or private server files. Releases are published by the maintainer after review. Submit only code you are entitled to contribute under the MIT license of this project.
