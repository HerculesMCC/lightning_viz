# Lightning Network Visualization

Un outil de visualisation du réseau Lightning développé en Rust. Ce projet permet de :
- Gérer des nœuds Bitcoin et Lightning en mode regtest
- Établir des connexions entre les nœuds
- Créer et gérer des canaux de paiement
- Visualiser le réseau de canaux avec une représentation graphique

## Prérequis

- Rust (édition 2021)
- Bitcoin Core (en mode regtest)
- Core Lightning (c-lightning)
- Graphviz (pour la visualisation)

### Installation des dépendances
Sur Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y pkg-config libfontconfig1-dev graphviz
Installation de Bitcoin Core et Core Lightning
(suivez les instructions d'installation officielles pour votre système)

## Configuration

1. Assurez-vous que Bitcoin Core est configuré en mode regtest
2. Configurez Core Lightning pour se connecter à votre nœud Bitcoin
3. Modifiez le fichier `config/default.toml` selon vos besoins

## Compilation et exécution
cargo build
cargo run
Afin de générer la visualisation
dot -Tpng lightning_network.dot -o network.png


## Fonctionnalités

- Création et gestion de nœuds Bitcoin et Lightning
- Établissement automatique de connexions entre les nœuds
- Création et gestion de canaux de paiement
- Visualisation du réseau avec Graphviz
- Support du mode regtest pour le développement

## Visualisation

Le projet génère :
- Un fichier DOT (`lightning_network.dot`)
- Une image PNG du réseau (`network.png`)
- Les nœuds sont représentés avec leurs alias et capacités
- Les canaux sont représentés avec leurs états et capacités


Kyllian Rousseleau
Nathanaël Desforges

