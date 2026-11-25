# Mon Budget

Application de **gestion de finances personnelles** : simple, rapide, 100 % locale.
Vos données restent sur votre machine, dans un fichier SQLite à côté du programme. Aucun
compte en ligne, aucune connexion requise.

Deux interfaces inclues, les mêmes données :

| Programme | Description |
|---|---|
| `mon-budget` | **Interface graphique** (egui) — tableau de bord, clics, couleurs Mistral |
| `mon-budget-cli` | **Interface console** — la version historique, utilisable au clavier |

## Fonctionnalités

- **Comptes** : courant, épargne, espèces… avec solde de départ et calcul automatique du solde
- **Transactions** : ajout, modification, suppression de revenus et dépenses (date, catégorie, note)
- **Catégories** prédéfinies (Alimentation, Loyer, Transport, Salaire…) — prêtes à l'emploi
- **Budgets mensuels** par catégorie : barre de progression, alerte à 90 % et en cas de dépassement
- **Rapport mensuel** : revenus, dépenses, reste (épargne), taux d'épargne, répartition par catégorie
- **Export CSV** : toutes les transactions, ouvrable directement dans Excel / LibreOffice Calc
- **Solde total** toujours visible
- **Interface graphique** aux teintes Mistral (dégradé jaune → rouge)

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
# Interface graphique
./target/release/mon-budget        # Windows : target\release\mon-budget.exe

# Interface console
./target/release/mon-budget-cli    # Windows : target\release\mon-budget-cli.exe

# Base de données personnalisée (console)
./mon-budget-cli mes-finances.db
```

L'interface graphique crée et lit le fichier `mon-budget.db` dans le dossier de lancement.
Pour sauvegarder vos données : copiez simplement ce fichier.

### Interface graphique

- **Tableau de bord** : comptes, soldes, mois en cours, dernières transactions
- **Transactions** : bouton « + Nouvelle transaction », navigation par mois, modification/suppression
- **Budgets** : bouton « + Nouveau budget », barres de progression colorées (orange → ambre à 90 % → rouge en dépassement)
- **Rapport mensuel** : revenus, dépenses, reste, taux d'épargne, comparaison des dépenses par catégorie
- **Export CSV** : un bouton, le fichier apparaît à côté de la base

### Interface console

Le menu s'utilise au clavier : tapez le numéro de l'option puis Entrée. La touche Entrée
seule permet de naviguer convenablement dans les listes.

## Compilation

```bash
cargo build --release
# → target/release/mon-budget (interface graphique)
# → target/release/mon-budget-cli (interface console)
```

Aucune dépendance système n'est nécessaire : SQLite est embarqué dans le binaire (`rusqlite`,
feature `bundled`). Linux : activez l'entité `x11` (inclus par défaut dans `Cargo.toml`).

## Structure du code

```
src/
├── lib.rs              Racine de la bibliothèque partagée
├── models.rs           Modèles (Account, Category, TransactionRow, BudgetLine, MonthSummary)
├── store.rs            Accès SQLite : schéma, requêtes, categories par défaut
├── export.rs           Export CSV partagé console/graphique
├── ui.rs               Assistants console (saisie clavier, euros, barres)
├── gui.rs              Interface graphique egui
├── main.rs             Binaire graphique
├── bin/cli.rs          Binaire console
└── menus/              Menus console (comptes, transactions, budgets, rapports, export)
```

Les montants sont stockés en **centimes entiers** (`i64`) pour éviter les erreurs d'arrondi.
Les tests : `cargo test` (6 tests).

## Idées d'évolution

- Transactions récurrentes (loyer, abonnements) générées automatiquement
- Import CSV
- Statistiques sur plusieurs mois (comparaison, tendance)
- Sauvegarde/restauration de la base
