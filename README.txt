Majher 1.9

Pierwsze uruchomienie i język:
- Przy pierwszym uruchomieniu program pyta o język interfejsu: polski, angielski, niemiecki albo rosyjski.
- Wybór jest zapisywany w pliku Majher.ini obok Majher.exe. Aby ponownie wybrać język, usuń Majher.ini.
- W głównym oknie można w dowolnej chwili zmienić język oraz wybrać motyw systemowy, jasny albo ciemny. Zmiany są zapisywane w Majher.ini obok programu.
- Motyw systemowy automatycznie dopasowuje wygląd do ustawienia aplikacji w Windows i reaguje na jego zmianę.
- Pola ścieżek, listy i dziennik korzystają ze standardowych kontrolek Windows oraz ciemnego tła w motywie ciemnym.
- Przyciski Wsparcie: Revolut i Wsparcie: PayPal otwierają oficjalne strony autora.
- Czytnik ekranu otrzymuje neutralne nazwy i opisy tych przycisków, bez odczytywania nazw użytkowników kont.

Minimalne wymagania:
- Windows 10 lub 11, 64-bitowy procesor x86-64.
- Folder, w którym Majher może zapisać Majher.ini oraz pobierane narzędzia; nie uruchamiaj programu bezpośrednio z archiwum ZIP.
- Połączenie z internetem do pierwszego pobrania brakujących narzędzi i sprawdzania aktualizacji. Do późniejszego dzielenia plików z już przygotowanym FFmpeg internet nie jest potrzebny.
- Karta NVIDIA nie jest wymagana. Akceleracja CUDA jest opcjonalna i zależy od zgodności karty oraz sterownika.

Program tnie długie pliki audio na mniejsze fragmenty. Jest zrobiony jako przenośna aplikacja Windows bez instalatora.

Uruchamianie:
1. Otwórz folder z programem.
2. Uruchom plik Majher.exe.
3. Dodaj jeden lub wiele plików audio.
4. Ustaw długość fragmentu w minutach.
5. Opcjonalnie wybierz folder wyjściowy. Puste pole tworzy folder z wynikami obok pliku wejściowego.
6. Wybierz format wyjściowy i profil zużycia zasobów komputera.
7. Wybierz bitrate i częstotliwość próbkowania pliku wynikowego.
8. Zostaw włączony tryb audiobook, jeżeli program ma wykrywać zapisane rozdziały oraz mówione zapowiedzi rozdziałów, części, ksiąg, tomów, kaset, stron i ścieżek.

Bitrate i częstotliwość:
- Bitrate można ustawić na 64, 96, 128, 160, 192, 256 albo 320 kb/s. Wpływa na MP3, M4A AAC oraz OPUS.
- Częstotliwość można pozostawić bez zmian albo ustawić dokładnie 44 100 Hz lub 48 000 Hz.
- OPUS obsługuje z podanych wartości 48 000 Hz. Przy wybraniu 44 100 Hz Majher wyświetli komunikat i poprosi o poprawienie ustawienia.
- Bitrate nie wpływa na tryb bez rekompresji, WAV ani FLAC. Wybrana częstotliwość działa dla formatów poddawanych rekompresji, w tym WAV i FLAC.

Postęp i zakończenie:
- Natywny pasek postępu Windows pokazuje postęp całego zadania od 0 do 100 procent.
- Podczas wykrywania ciszy i cięcia pasek korzysta z rzeczywistego czasu raportowanego przez FFmpeg.
- Pasek zgłasza zmiany wartości przez MSAA. NVDA może sygnalizować je dźwiękiem zgodnie z ustawieniami pasków postępu w NVDA.
- Po poprawnym zakończeniu pojawia się okno Sukces z liczbą przetworzonych plików wejściowych i utworzonych części.
- To samo okno pyta, czy przejść do folderu z pociętą książką. Przycisk Tak otwiera dokładny folder wynikowy, a przycisk Nie zamyka komunikat bez otwierania Eksploratora.
- Przy cięciu wielu książek Majher może po potwierdzeniu otworzyć ich osobne foldery wynikowe.

Dostępność:
- Interfejs używa standardowych kontrolek Windows: przycisków, pól edycji, list, list rozwijanych i pola statusu.
- Okno działa jak dialog, więc po uruchomieniu fokus trafia na przycisk Dodaj pliki, a klawisz Tab przechodzi po kontrolkach.
- Najważniejsze kontrolki mają ustawione dostępne nazwy i opisy przez MSAA, co jest obsługiwane przez NVDA i JAWS.
- Pole statusu jest zwykłym polem tekstowym, więc można wejść na nie klawiszem Tab i odczytać komunikaty czytnikiem ekranu.
- Program nie używa niestandardowo rysowanych kontrolek.

Zasoby komputera:
- Niskie: najmniej wątków i niski priorytet procesu FFmpeg.
- Normalne: umiarkowana liczba wątków i niższy priorytet.
- Szybkie: więcej wątków i normalny priorytet.

Formaty:
Program korzysta z FFmpeg, dlatego obsługuje typowe formaty audio, m.in. MP3, M4A, M4B, AAC, WAV, FLAC, OGG, OPUS, WMA, AIFF, AC3, MKA, WEBM i MP4 z audio.

Tryb audiobook:
- Rozdziały zapisane w metadanych pliku mają pierwszeństwo i są używane bez rozpoznawania mowy.
- Gdy plik nie ma rozdziałów, program wyszukuje dłuższe pauzy po odszumieniu nagrania i sprawdza ich otoczenie lokalnym modelem Whisper.
- Rozpoznawane są polskie zapowiedzi, m.in. rozdział, część, księga, tom, kaseta, strona A lub B, ścieżka, prolog i epilog.
- Ustawiona liczba minut jest w tym trybie maksymalną długością awaryjną. Wykryte krótsze rozdziały pozostają osobnymi plikami, a bardzo długi rozdział może zostać dodatkowo podzielony przy pobliskiej pauzie.
- Jeżeli żadna zapowiedź nie zostanie rozpoznana, program zachowuje dotychczasowe bezpieczne cięcie według długości i pobliskich pauz.
- Program porównuje długość kontenera i strumieni audio. Pełne skanowanie dodatkowo koryguje błędną długość zapisaną w nagłówku audiobooka.

Model rozpoznawania:
- Przy pierwszym użyciu Majher pyta o zgodę na pobranie potrzebnych składników do tools\chapter-detector obok Majher.exe.
- Model ggml-small-q5_1.bin zajmuje 190 085 487 bajtów. Pobrany plik jest sprawdzany sumą SHA-256.
- Bez karty NVIDIA pobierany jest model i mały silnik CPU, łącznie około 190 MB.
- Po wykryciu NVIDIA program pozwala wybrać pobranie samodzielnej paczki CUDA albo pozostanie przy mniejszym silniku CPU. Jeżeli wybrane GPU lub sterownik okażą się niezgodne, automatycznie wraca do CPU.
- Od wersji 1.6 Majher najpierw przygotowuje wszystkie krótkie próbki, a następnie przekazuje je do jednego wspólnego procesu CUDA. Model jest ładowany do pamięci karty tylko raz, działa Flash Attention, a program przed analizą potwierdza, że backend NVIDIA faktycznie się uruchomił.
- Moc i procent użycia GPU zależą od karty, sterownika, liczby próbek oraz małego skwantowanego modelu. Nie każda karta osiągnie swoje maksymalne TGP, ale analiza nie powinna już tracić czasu na ponowne ładowanie modelu dla każdej próbki.
- Model działa lokalnie. Nagranie ani rozpoznany tekst nie są wysyłane do internetu.
- Model, silniki oraz pliki tymczasowe pobierania można usunąć razem z katalogiem tools\chapter-detector. Przy kolejnym użyciu program zaproponuje ich ponowne pobranie.

Stabilność i zatrzymywanie:
- Dziennik FFmpeg jest przechowywany w małym buforze, dlatego wielogodzinny plik nie powinien już stopniowo zajmować pamięci programu.
- Przycisk Zatrzymaj teraz kończy również bieżące wykrywanie ciszy, rozpoznawanie modelem, pobieranie modelu lub cięcie.
- Nieoczekiwany błąd zadania jest pokazywany w dzienniku, a okno wraca do normalnego stanu zamiast zamknąć całą aplikację.
- Fokus jest ustawiany automatycznie tylko przy uruchomieniu i zakończeniu zadania. Podczas nawigacji program nie wymusza zmiany kontrolki, dzięki czemu nie walczy z NVDA ani JAWS.
- Lekkie zdarzenia postępu regularnie odświeżają informację dla czytnika ekranu bez przełączania okna programu.

FFmpeg:
Program najpierw szuka ffmpeg.exe i ffprobe.exe zainstalowanych w systemie. Jeżeli ich nie znajdzie, korzysta z kopii w tools\ffmpeg obok pliku programu. Jeśli nie ma również kopii lokalnej, sam tworzy folder tools, pobiera najnowszą stabilną paczkę Windows Essentials z gyan.dev, sprawdza jej opublikowaną sumę SHA-256 i ją rozpakowuje.
Po wykryciu kompletnego FFmpeg w systemie Majher wyświetla wyraźny komunikat, że użyje wersji systemowej i niczego nie będzie pobierał.

Aktualizacje:
- Od wersji 1.7 nowe wydania są publikowane wyłącznie przez GitHub Releases; własny serwer autora nie jest używany przez aktualizator.
- Program automatycznie sprawdza dostępność nowszej wersji przy każdym uruchomieniu.
- Przycisk Sprawdź aktualizacje porównuje obecną wersję programu z najnowszym wydaniem GitHub Releases.
- Aktualizacje oraz zweryfikowane pliki konfiguracyjne składników są pobierane wyłącznie z publicznego repozytorium proszczok2002/majher na GitHubie.
- Aktualizacje korzystają ze statycznych plików version.txt, changelog.txt, Majher.exe i Majher.exe.sha256 dołączonych do najnowszego wydania.
- Po znalezieniu nowszej wersji program pobiera ją, sprawdza sumę SHA-256 i pokazuje listę zmian.
- Po potwierdzeniu Majher zamyka się, tworzy tymczasową kopię .bak, podmienia program i uruchamia nową wersję. Po udanym uruchomieniu kopia .bak jest automatycznie usuwana.
- Adres techniczny aktualizacji nie jest pokazywany w interfejsie ani w dzienniku.
