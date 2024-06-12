# Mailing List

## Introduction
Ce projet est une application de gestion de mailing list développée en Rust utilisant les frameworks Actix-web et Axum. Il permet aux utilisateurs de s'inscrire, de se connecter, et de gérer des listes de diffusion (mailing lists).

## Fonctionnalités principales
- Inscription et connexion des utilisateurs
- Création, visualisation et suppression de listes de diffusion
- Vérification des emails et gestion des abonnés aux listes

## Instructions d'installation et d'utilisation

### Prérequis
- [Rust](https://www.rust-lang.org/tools/install) installé sur votre machine
- [MongoDB](https://www.mongodb.com/try/download/community) pour la base de données

### Installation
Clonez le dépôt du projet :
```sh
git clone https://github.com/jeremiekunkela/mailing_list.git
cd votre-projet
```

Installez les dépendances :
```sh
cargo build
```

### Configuration
Créez un fichier `.env` à la racine du projet pour configurer les variables d'environnement nécessaires :
```
MONGO_URI=mongodb://localhost:27017
DATABASE_NAME=mailing_list
```

### Lancer le programme
Pour démarrer l'application :
```sh
cargo run
```

## API Routes
Les routes principales de l'API sont :

- `POST /signup` : Inscription d'un utilisateur
- `POST /login` : Connexion d'un utilisateur
- `GET /mailing_lists` : Récupérer toutes les listes de diffusion
- `GET /user/{user_id}/mailing_lists` : Récupérer les listes de diffusion d'un utilisateur
- `POST /mailing_list` : Créer une nouvelle liste de diffusion
- `DELETE /mailing_list/{id}` : Supprimer une liste de diffusion

## Exemple de requêtes
### Inscription
```sh
curl -X POST http://localhost:8080/signup -H "Content-Type: application/json" -d '{"username":"testuser","email":"test@example.com","password":"password123"}'
```

### Connexion
```sh
curl -X POST http://localhost:8080/login -H "Content-Type: application/json" -d '{"username":"testuser","password":"password123"}'
```
