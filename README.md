# Mon Budget

Application de **gestion de finances personnelles** en Rust : simple, rapide, 100 % locale.
Vos données restent sur votre machine, dans un fichier SQLite à côté du programme. Aucun
compte en ligne, aucune connexion requise.

## Fonctionnalités

- **Comptes** : courant, épargne, espèces… avec solde de départ et calcul automatique du solde
- **Transactions** : ajout, modification, suppression de revenus et dépenses (date, catégorie, note)
- **Catégories** prédéfinies (Alimentation, Loyer, Transport, Salaire…) — prêtes à l'emploi
- **Budgets mensuels** par catégorie : barre de progression, alerte à 90 % et en cas de dépassement
- **Rapport mensuel** : revenus, dépenses, reste (épargne), taux d'épargne, répartition par catégorie
- **Export CSV** : toutes les transactions, ouvrable directement dans Excel / LibreOffice Calc
- **Solde total** affiché en permanence sur le menu principal
- **Interface en couleurs** aux teintes Mistral (dégradé jaune → rouge) dans un terminal 24 bits,
  avec désactivation automatique si la sortie n'est pas interactive

## Utilisation

```bash
# Lancer (par exemple avec le dossier du programme comme répertoire courant)
./mon-budget

# Base de données personnalisée
./mon-budget mes-finances.db
```

Le menu s'utilise au clavier : tapez le numéro de l'option puis Entrée. La touche Entrée
seule permet de naviguer convenablement dans les listes.

## Compilation

```bash
cargo build --release
# → target/release/mon-budget
```

Aucune dépendance système n'est nécessaire : SQLite est embarqué dans le binaire via
`rusqlite` (feature `bundled`).

## Structure du code

```
src/
├── main.rs            Point d'entrée : ouverture de la base puis menu principal
├── models.rs          Modèles (Account, Category, TransactionRow, BudgetLine, MonthSummary)
├── store.rs           Accès SQLite : schéma, requêtes, categories par défaut
├── ui.rs              Saisie clavier (montants, dates, sélection) et affichage (euros, barres)
└── menus/
    ├── mod.rs         Menu principal
    ├── accounts.rs    Comptes
    ├── transactions.rs  Transactions
    ├── budgets.rs     Budgets mensuels
    ├── reports.rs     Rapport mensuel
    └── tools.rs       Export CSV
```

Les montants sont stockés en **centimes entiers** (`i64`) pour éviter les erreurs d'arrondi.

## Idées d'évolution

- Transactions récurrentes (loyer, abonnements) générées automatiquement
- Import CSV
- Statistiques sur plusieurs mois (comparaison, tendance)
- Sauvegarde/restauration de la base
