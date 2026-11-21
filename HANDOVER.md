# Übergabe: neue Rätsel für die Sudoku-Seite

Für Simon, der die Website betreut. Hier steht, was sich geändert hat, welche Dateien du brauchst
und wie du die Änderungen in die Live-Seite übernimmst.

## Kurz gesagt

* `data/puzzles.csv`: **3.100 bewertete Rätsel mit Lösung.** Schwer 500, Experte 800, die neue
  Stufe Meister 800, Extrem 1.000. Jedes Rätsel ist geprüft: genau eine Lösung, und keine
  Vorgabe lässt sich weglassen.
* `index.html`: die ursprüngliche Seite mit allen Änderungen und den eingebetteten Rätseln.
* `scripts/check_puzzles.mjs` und `scripts/csv_to_page.mjs`: reines Node ab Version 18, ohne
  Installation.
* `RATINGS.md` (Englisch): was die Bewertungen bedeuten und warum 9,0 gegenüber 11,9 ein riesiger
  Schritt ist, 2,6 gegenüber 4,4 aber nicht.

## Die Rätseldatei

`data/puzzles.csv` (UTF-8, durch Kommas getrennt, wo nötig in Anführungszeichen):

| Spalte | Beispiel | Bedeutung |
|--------|----------|-----------|
| level | `extrem` | `schwer`, `experte`, `meister`, `extrem` |
| puzzle | `98.7.....7.....6...` | 81 Zeichen, Zeile für Zeile, `.` = leer |
| solution | `987654321246...` | 81 Ziffern |
| rating | `11.9` | Sudoku-Explainer-Bewertung, eine Nachkommastelle, mit Punkt |
| source | `classic` | `classic` bekanntes Rätsel mit Namen, `new` in diesem Projekt neu gefunden, `ranking-list` Forums-Rangliste der schwersten Sudokus, `generated` hier erzeugt |
| name | `„Champagne Dry“` | bei Klassikern die Wendung, wie sie in „Es beruht auf …“ steht; bei der Rangliste die Kennung aus dem Forum (wird auf der Seite nicht angezeigt); sonst leer |

Bewertungsbereiche je Stufe: Schwer 2,6–4,4, Experte 4,5–8,9, Meister 9,0–11,3, Extrem 11,5–11,9.

## Übernahme in die Live-Seite

Das Repository ist öffentlich: https://github.com/lllAlexanderlll/sudoku

**Variante A: Die Live-Seite ist noch die Fassung, die Alexander mir gegeben hat.** Das ist der
erste Commit dieses Repositorys, `69ba738`. Dann ersetzt du sie durch `index.html`. Zum Prüfen
vergleichst du die Live-Datei mit `git show 69ba738:index.html`. Sind beide gleich, bist du
damit fertig.

**Variante B: Die Live-Seite hat sich seitdem geändert.** Dann überträgst du die Änderungen aus
der Liste unten. Der Prompt am Ende dieser Datei ist so geschrieben, dass Claude das erledigen
kann.

## Änderungen gegenüber der ursprünglichen Seite

Bis auf die Liste der Stufen im HTML liegt alles im `<script>`-Teil.

1. **Ab Schwer kommen die Rätsel aus eingebetteten Listen, statt im Browser erzeugt zu
   werden.** Der Block zwischen `// BANK-START` und `// BANK-END` ersetzt die Liste `FAMOUS`.
   Er enthält zwei Dinge:
   * `NAMED`: die Namen der Klassiker.
   * `BANK`: je Stufe ein String mit einer Zeile pro Rätsel. Eine Zeile besteht aus 81 Zeichen
     Rätsel, drei Ziffern Bewertung (`115` steht für 11,5) und einem Buchstaben für die Herkunft
     (`F`, `N`, `R` oder `G`).

   *Warum:* Die alten Rätsel für Schwer und Experte brauchten bis zu 12 Sekunden zum Erzeugen.
   Ihre Schwierigkeit war nur „mit einfachen Schritten nicht lösbar“, und das kann alles
   zwischen 2,6 und 11 sein. Jetzt ist jedes Rätsel bewertet und sofort da.
2. **Neue Stufe „Meister“ (9,0–11,3)** zwischen Experte und Extrem: ein neuer Radio-Button und
   ein Eintrag in `LEVELS`. Das alte Extrem mit 17 Klassikern mischte 10,5 und 11,9.
3. **Keine Wiederholungen je Stufe:** `nextFromBank(level)` ersetzt `nextFamous()`. Die Funktion
   speichert in `localStorage` unter `sudoku-ansage-stapel-v2-<stufe>` nur einen Zufallsstartwert
   und eine Position. Die gemischte Reihenfolge baut sie daraus jedes Mal neu auf.
   * Der alte Schlüssel `sudoku-ansage-extrem-stapel-v1` wird nicht mehr benutzt und kann
     bleiben.
   * Der gespeicherte Spielstand (`sudoku-ansage-game-v2`) und die Einstellungen sind
     unverändert.
4. **Die Bewertung wird angesagt:** Der Text „Es beruht auf …“ nennt jetzt die Herkunft des
   Rätsels und endet mit „(Bewertung 11,5)“.
5. **Die Beschreibungen der Stufen** zeigen den Bewertungsbereich und die Anzahl der Rätsel. Beim
   Start werden sie aus der Liste befüllt (`data-range`, `data-bank`), statt fest im Text zu
   stehen.
6. **„Bitte warten“ nur noch bei Leicht und Mittel**, den beiden Stufen, die weiterhin im Browser
   erzeugt werden.
7. **Entfernter Code:** `eliminate()` und das mehrstufige `logicSolve()`. Beides brauchte nur die
   alte Erzeugung von Schwer und Experte. `logicSolve()` prüft jetzt nur noch Singles, für Leicht
   und Mittel.
8. **Doppeltes Rätsel entfernt:** „eleven #1“ und „eleven #3“ waren dasselbe Rätsel. Das hat ein
   exakter Isomorphie-Test über alle Umstellungen bestätigt. Es gibt also 16 Klassiker. Drei
   davon sind mit 10,5–10,6 bewertet und stehen jetzt unter Meister: AI Escargot, Platinum Blonde
   und Arto Inkalas Rätsel von 2012.

Die Seite wächst von 95 KB auf 352 KB. Das lädt problemlos, auch auf dem Handy.

## Empfehlungen

**Das Mischen beibehalten.** Es gibt zwei Arten, und beide waren schon in der ursprünglichen
Seite:

* *Gemischte Reihenfolge ohne Wiederholung* (Änderung 3): beibehalten. Sonst bekäme Alexanders
  Mutter Wiederholungen oder immer dieselbe Reihenfolge.
* *Jedes Rätsel vor dem Vorlesen umstellen* (`transform()`: Ziffern tauschen, Zeilen, Spalten
  und Bänder tauschen, eventuell spiegeln). Das Original machte das für Extrem, jetzt gilt es für
  alle Stufen ab Schwer.
  * Die Schwierigkeit bleibt dabei gleich.
  * Ein bekanntes Rätsel lässt sich so nicht an seinen Vorgaben wiedererkennen.
  * Es kostet nichts. Die Lösung des umgestellten Rätsels berechnet die Seite danach mit
    `solve()`, das dauert selbst bei den schwersten Rätseln unter 50 ms (gemessen an allen
    Rätseln von Meister und Extrem).
  * Weglassen würde nur etwa 15 Zeilen sparen.

**Die Rätsel einbetten, die CSV nicht per `fetch()` laden.** So bleibt die Seite eine einzige
Datei. Sie funktioniert dann auch offline und wenn man sie direkt von der Festplatte öffnet
(`file://`); dort schlägt `fetch()` fehl. Die CSV ist die Quelle, den eingebetteten Block
erzeugst du daraus neu (siehe unten).

**Keine Prüfung in der Seite.** Jedes Rätsel wurde beim Zusammenstellen der Listen geprüft, und
die Seite berechnet die Lösung ohnehin selbst. **Lass die Prüfung aber laufen, sobald sich die
Rätseldaten oder der Code zum Einbetten ändern.** Ein einziges falsches Zeichen macht ein Rätsel
unlösbar oder mehrdeutig, und das merkt sie erst nach langem Rätseln:

```bash
node scripts/check_puzzles.mjs data/puzzles.csv index.html
```

Das Skript prüft zwei Dinge:
* Jedes Rätsel der CSV hat genau eine Lösung, passt zur Spalte `solution` und ist minimal. Die
  Bewertung passt zur Stufe, und kein Rätsel kommt doppelt vor.
* Die Seite enthält genau die Rätsel der CSV, mit denselben Stufen und Bewertungen.

Es dauert etwa 2 Minuten und endet mit `ALL OK`. Bettet die Live-Seite die Rätsel anders ein,
lass Claude den Seitenteil des Skripts anpassen.

## Rätsel später ändern

* **Rätsel entfernen oder verschieben:** Bearbeite `data/puzzles.csv`, dann:

```bash
node scripts/csv_to_page.mjs data/puzzles.csv index.html
```

```bash
node scripts/check_puzzles.mjs data/puzzles.csv index.html
```

* **Rätsel hinzufügen:** Neue Rätsel brauchen eine Bewertung mit Sudoku Explainer. Wie die
  Listen erzeugt und bewertet wurden, steht in `README.md` unter „Reproducing“ (Englisch).

## Prompt für Claude (Variante B)

> In diesem Repository liegt unsere barrierefreie Sudoku-Seite. Ich möchte die neuen Rätsellisten
> aus https://github.com/lllAlexanderlll/sudoku übernehmen.
>
> 1. Lies dort HANDOVER.md.
> 2. Vergleiche die `index.html` aus dem ersten Commit (unsere alte Fassung) mit der aktuellen
>    `index.html`.
> 3. Übertrage dieselben Änderungen auf unsere Seite: den BANK-Block, `nextFromBank`, die Stufe
>    Meister, die Bewertung im Text „Es beruht auf …“, die Beschreibungen der Stufen, die Änderung
>    bei „Bitte warten“ und den entfernten Code. Alles andere, was wir seitdem geändert haben,
>    bleibt erhalten. `transform()` und die Reihenfolge ohne Wiederholung bleiben ebenfalls.
> 4. Erzeuge den BANK-Block mit `node scripts/csv_to_page.mjs data/puzzles.csv <unsere Seite>`,
>    nicht von Hand.
> 5. Führe danach `node scripts/check_puzzles.mjs data/puzzles.csv <unsere Seite>` aus.
> 6. Öffne die Seite und prüfe, dass alle sechs Stufen ohne Fehler in der Konsole ein Rätsel
>    starten.
