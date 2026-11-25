# Mon Budget

Application de **gestion de finances personnelles** : simple, ludique, 100 % locale.
Vos données restent sur votre machine, dans un fichier SQLite à côté du programme. Aucun
compte en ligne, aucune connexion requise.

Trois interfaces inclues, les mêmes données :

| Programme | Description |
|---|---|
| `mon-budget` | **Application web locale** — s'ouvre dans le navigateur, boutons et menus comme un site internet |
| `mon-budget-egui` | Interface graphique en fenêtre native (egui) |
| `mon-budget-cli` | Interface console historique |

## Fonctionnalités

- **Comptes** : courant, épargne, espèces… avec solde de départ et calcul automatique du solde
- **Transactions** : ajout **en un clic** (boutons rapides « + Dépense », « + Revenu », raccourcis
  par catégorie), modification, suppression (revenus et dépenses, date, catégorie, note)
- **Catégories** prédéfinies (Alimentation, Loyer, Transport, Salaire…) — prêtes à l'emploi
- **Budgets mensuels** par catégorie : barre de progression, alerte à 90 % et en cas de dépassement
- **Rapport mensuel** : revenus, dépenses, reste (épargne), taux d'épargne, répartition par catégorie
- **Export CSV** : téléchargement direct, ouvrable dans Excel / LibreOffice Calc
- **Solde total** toujours visible
- Couleurs aux teintes **Mistral** (dégradé jaune → rouge)

## Installation

Il suffit d'installer Rust une fois :

- Windows : téléchargez [rustup-init.exe](https://win.rustup.rs/x86_64) (acceptez l'installation
  des Build Tools C++ si proposée)
- Linux / macOS : `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

Puis :

```bash
git clone https://github.com/christopher-lhostis/Applifinance.git
cd Applifinance
cargo build --release
```

## Utilisation

```bash
# Application web locale (recommandée) : ouvre http://127.0.0.1:8630 dans votre navigateur
./target/release/mon-budget            # Windows : target\release\mon-budget.exe

# Interface en fenêtre native
./target/release/mon-budget-egui

# Interface console
./target/release/mon-budget-cli [ma-base.db]
```

L'application web crée et lit le fichier `mon-budget.db` dans le dossier de lancement.
Pour sauvegarder vos données : copiez simplement ce fichier.
Pour arrêter : fermez la fenêtre du terminal (ou Ctrl+C).

### L'interface web

- **Boutons de menu** en haut pour changer de section (comme sur un site internet)
- **Tableau de bord** : comptes, soldes, résumé du mois, dernières transactions
- **Ajout en un clic** : boutons « + Dépense », « + Revenu » et raccourcis par catégorie
  (Courses, Transport, Loisirs…) qui ouvrent un formulaire pré-rempli : il n'y a que
  le montant à saisir
- **Transactions** : navigation de mois en mois (‹ ›), modification/suppression sur chaque ligne
- **Budgets** : formulaire simple, barres de progression au code couleur (orange → ambre à 90 % → rouge en dépassement)
- **Rapport mensuel** : taux d'épargne, comparaison des dépenses par catégorie
- **Export CSV** : un bouton, le fichier se télécharge

### Interfaces egui et console

- egui : navigation par boutons dans le panneau de gauche, mêmes fonctionnalités
- console : tapez le numéro de l'option puis Entrée ; Entrée seule navigue/arrière

## Compilation

```bash
cargo build --release
# → target/release/mon-budget (web, par défaut)
# → target/release/mon-budget-egui (fenêtre native)
# → target/release/mon-budget-cli (console)
```

Aucune dépendance système : SQLite est embarqué (`rusqlite`, feature `bundled`) et la page
d'interface est incluse dans le binaire. Linux : feature `x11` activée pour la variante egui.

## Structure du code

```
src/
├── lib.rs              Racine de la bibliothèque partagée
├── models.rs           Modèles (Account, Category, TransactionRow, BudgetLine, MonthSummary)
├── store.rs            Accès SQLite : schéma, requêtes, categories par défaut
├── export.rs           Génération CSV partagée
├── web.rs              Serveur local + API JSON
├── static/index.html   Page web (boutons, cartes, formulaires)
├── gui.rs              Interface fenêtre native (egui)
├── main.rs             Binaire web (par défaut)
├── bin/egui.rs         Binaire egui
├── bin/cli.rs          Binaire console
└── menus/              Menus console
```

Les montants sont stockés en **centimes entiers** (`i64`) pour éviter les erreurs d'arrondi.
Les tests : `cargo test` (8 tests).

## Idées d'évolution

- Transactions récurrentes (loyer, abonnements) générées automatiquement
- Import CSV
- Statistiques sur plusieurs mois (comparaison, tendance)
- Sauvegarde/restauration de la base
