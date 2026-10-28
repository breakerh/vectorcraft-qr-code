# QR-code maken — VectorCraft-plugin

Zet tekst of een URL om naar een bewerkbare vector-QR-code in het open VectorCraft-document. Getest met VectorCraft 0.17.0.

[Download de plugin](QR-code-maken.wasm) (open het bestand in VectorCraft om te installeren).

## Gebruik

1. Open je document en hef bij voorkeur de bestaande selectie op.
2. Kies **Object → Plug-ins → QR-code maken…**.
3. Vul je tekst of volledige URL in, bijvoorbeeld `https://example.com`.
4. Kies het formaat in millimeters (inclusief de scanmarge) en de foutcorrectie. M is een goede standaard.
5. Laat de witte achtergrond aan voor een betrouwbare lichte scanmarge, of schakel hem uit als de ondergrond al egaal wit is.
6. Klik **OK**. De code verschijnt midden op het tekenbord als een zwart vectorpad met meerdere gesloten deelpaden, plus eventueel een apart wit achtergrondvlak.

Je kunt de objecten groeperen, verplaatsen en proportioneel schalen. De invoeging kan met één keer Ongedaan maken worden teruggedraaid. Als er objecten geselecteerd waren, bewaart de plugin die ongewijzigd; VectorCraft kan ze samen met de nieuwe QR-code geselecteerd houden.

De code is statisch: opnieuw openen van de plugin maakt een nieuwe code. Voor een andere URL maak je een nieuwe code en verwijder je de vorige.

## Installeerbestand

Open `QR-code-maken.wasm` via Bestand → Openen of Object → Plug-ins → Plug-in installeren. De plugin gebruikt VectorCrafts grotere rekenbudget voor langere teksten. Bevestig de melding daarover tijdens installatie. Hij rekent lokaal, draait op de achtergrond en kan worden geannuleerd. Gebruik voor deze plugin de installatieoptie; een map met extra plugins slaat plugins met dit budget over.



## Eigenschappen

- Volledig offline; geen netwerk, account, abonnement of trackingdienst.
- Bewerkbare vectorpaden, zonder rasterafbeelding.
- Formaat 10–1000 mm, inclusief vier QR-modules scanmarge rondom.
- Foutcorrectie L, M, Q of H. De encoder kan foutcorrectie verhogen als dezelfde QR-afmetingen dat toelaten.
- Unicode-tekst wordt als UTF-8 verwerkt; ingevoerde spaties worden behouden.
- QR-versie wordt automatisch gekozen. Te lange invoer krijgt een foutmelding; capaciteit hangt af van tekst en foutcorrectie. Het tekstveld neemt maximaal 4096 tekens aan.
- Kleine afdrukformaten met veel tekst geven zeer kleine modules. Controleer de uiteindelijke afdruk met een scanner.

## Validatie

- Drie Rust-tests: behoud bestaande selectie/formaat/positie, foutmeldingen, Unicode/transparante achtergrond.
- 24 uitvoeringen van het echte WASM-bestand teruggelezen met ZXing, byte voor byte gelijk aan de invoer. Onder andere URL-queryparameters, Unicode, spaties en 1000 bytes tekst, op alle vier niveaus.
- Getest in wasmi 2.0.0 met het rekenbudget van de host, tot 2953 bytes bij niveau L; geschikte langere gevallen slagen en overschrijding van QR-capaciteit geeft een foutmelding.
- Geen WASM-hostimports.
- Native VectorCraft: installatie, dialoog en vectorvoorvertoning met de standaard-URL gecontroleerd. Definitieve build geïnstalleerd en bestandshash geverifieerd. Een volledige hertest van de definitieve dialoog staat nog open.

## Zelf bouwen

Vereist Rust met het doelplatform wasm32-unknown-unknown:

```sh
rustup target add wasm32-unknown-unknown
cargo test --locked
cargo build --locked --release --target wasm32-unknown-unknown
```

Resultaat: `target/wasm32-unknown-unknown/release/vectorcraft_qr_code.wasm`.

Plugin-ABI: https://github.com/storytold/vectorcraft/blob/main/docs/plugins.md
QR-encoder: https://github.com/nayuki/QR-Code-generator
