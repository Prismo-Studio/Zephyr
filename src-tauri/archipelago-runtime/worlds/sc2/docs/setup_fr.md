# Guide d'installation du _StarCraft 2 Randomizer_

Ce guide contient les instructions pour installer et dépanner le client de _StarCraft 2 Archipelago_, ainsi que des
indications pour obtenir un fichier de configuration de _StarCraft 2 Archipelago_ et comment modifier ce dernier.

## Logiciels requis

- [_StarCraft 2_](https://starcraft2.com/en-us/)
  - Bien que _StarCraft 2 Archipelago_ supporte les quatre campagnes, elles ne sont pas obligatoires pour jouer au
    _randomizer_.
    Si vous ne possédez pas certaines campagnes, il vous suffit de les exclure dans le fichier de configuration de
    votre monde.
- [La version la plus récente d'Archipelago](https://github.com/ArchipelagoMW/Archipelago/releases)

## Comment est-ce que j'installe ce _randomizer_?

1. Installer _StarCraft 2_ et Archipelago en suivant les instructions indiquées dans les liens précédents. Le client de
   _StarCraft 2 Archipelago_ est téléchargé par le programme d'installation d'Archipelago.
   - Les utilisateurs de Linux devraient aussi suivre les instructions qui se retrouvent à la fin de cette page
     (["Exécuter sous Linux"](#exécuter-sous-linux)).
   - Notez que votre jeu _StarCraft 2_ doit être en anglais pour fonctionner avec Archipelago.
2. Exécuter `ArchipelagoStarcraft2Client.exe`.
   - Uniquement pour cette étape, les utilisateurs de macOS devraient plutôt suivre les instructions qui se trouvent à
     ["Exécuter sous macOS"](#exécuter-sous-macos).
3. Dans le client de _StarCraft 2 Archipelago_, écrire la commande `/download_data`. Cette commande va lancer
   l'installation des fichiers qui sont nécessaires pour jouer à _StarCraft 2 Archipelago_.

## Où est-ce que j'obtiens le fichier de configuration (i.e., le _yaml_) pour ce jeu?

Un fichier dans le format _yaml_ est utilisé pour communiquer à Archipelago comment vous voulez que votre jeu soit
_randomized_.
Ce dernier est nécessaire même si vous voulez utiliser les options par défaut.
L'approche usuelle pour générer un _multiworld_ consiste à avoir un fichier _yaml_ par monde.

Il y a trois approches pour obtenir un fichier _yaml_ pour _StarCraft 2 Randomizer_:

- Vous pouvez aller à la page [_Player options_](/games/Starcraft%202/player-options) qui vous permet de définir vos
  choix via une interface graphique et ensuite télécharger le _yaml_ correspondant à ces choix.
- Vous pouvez obtenir le modèle de base en le téléchargeant à la page
  [_Player options_](/games/Starcraft%202/player-options) ou en cliquant sur _Generate template_ après avoir exécuté le
  _Launcher_ d'Archipelago (i.e., `ArchipelagoLauncher.exe`). Ce modèle de base inclut une description pour chacune des
  options et vous n'avez qu'à modifier les options dans un éditeur de texte de votre choix.
- Vous pouvez demander à quelqu'un d'autre de partager un de ces fichiers _yaml_ pour l'utiliser ou l'ajuster à vos
  préférences.

Prenez soin de vous rappeler du nom de joueur que vous avez inscrit dans la page à options ou dans le fichier _yaml_
puisque vous en aurez besoin pour vous connecter à votre monde!

Si vous désirez des informations et/ou instructions générales sur l'utilisation d'un fichier _yaml_ pour Archipelago,
veuillez consulter [_Creating a YAML_](/tutorial/Archipelago/setup/en#creating-a-yaml).

### Questions récurrentes à propos du fichier _yaml_

#### Comment est-ce que je sais que mon _yaml_ est bien défini?

La manière la plus simple de valider votre _yaml_ est d'utiliser le
[système de validation](/check) du site web.

Vous pouvez aussi le tester en tentant de générer un _multiworld_ avec votre _yaml_.
Pour faire ça, sauvegardez votre _yaml_ dans le dossier `Players/` de votre installation d'Archipelago et exécutez
`ArchipelagoGenerate.exe`.
Si votre _yaml_ est bien défini, vous devriez voir un nouveau fichier, avec l'extension `.zip`, apparaître dans le
dossier `output/` de votre installation d'Archipelago.
Il est recommandé de lancer `ArchipelagoGenerate.exe` via un terminal afin que vous puissiez voir les messages générés
par le logiciel, ce qui va inclure toutes erreurs qui ont eu lieu et le nom de fichier généré.
Si vous n'appréciez pas le fait d'utiliser un terminal, vous pouvez aussi regarder le fichier _log_ qui va être produit
dans le dossier `logs/`.

#### À quoi sert l'option _Progression Balancing_?

Pour _StarCraft 2_, cette option ne fait pas grand-chose.
Il s'agit d'une option d'Archipelago permettant d'équilibrer la progression des mondes en interchangeant les _items_
dans les _spheres_.
Si le _Progression Balancing_ d'un monde est plus grand que ceux des autres, les _items_ de progression de ce monde ont
plus de chance d'être obtenus tôt et vice-versa si sa valeur est plus petite que celle des autres mondes.
Cependant, _StarCraft 2_ est beaucoup plus permissif en termes d'_items_ qui permettent de progresser, ce réglage à
donc peu d'influence sur la progression dans _StarCraft 2_.
Vu qu'il augmente le temps de génération d'un _MultiWorld_, nous recommandons de le désactiver, c-à-d le définir à
zéro, pour _StarCraft 2_.

#### Comment est-ce que je définis une liste d'_items_, e.g. pour l'option _excluded items_?

Vous pouvez lire sur la syntaxe des conteneurs dans le format _yaml_ à la page
[_YAML specification_](https://yaml.org/spec/1.2.2/#21-collections).
Pour les listes, chaque _item_ doit être sur sa propre ligne et doit être précédé par un trait d'union.

```yaml
excluded_items:
  - Battlecruiser
  - Drop-Pods (Kerrigan Ability)
```

Une liste vide est représentée par une paire de crochets: `[]`.
Il s'agit de la valeur par défaut dans le modèle de base, ce qui devrait vous aider à apprendre à utiliser cette
syntaxe.

#### Comment est-ce que je fais pour avoir des _items_ dès le départ?

L'option _starting inventory_ est un _map_ et non une liste.
Ainsi, elle permet de spécifier le nombre de chaque _item_ avec lequel vous allez commencer.
Sa syntaxe consiste à indiquer le nom de l'_item_, suivi par un deux-points, puis par un espace et enfin par le nombre
désiré de cet _item_.

```yaml
start_inventory:
  Micro-Filtering: 1
  Additional Starting Vespene: 5
```

Un _map_ vide est représenté par une paire d'accolades: `{}`.
Il s'agit de la valeur par défaut dans le modèle de base, ce qui devrait vous aider à apprendre à utiliser cette
syntaxe.

#### Comment est-ce que je fais pour connaître le nom des _items_ et des _locations_ dans _StarCraft 2 Archipelago_?

La page [_datapackage_](/datapackage) d'Archipelago liste l'ensemble des _items_ et des _locations_ de tous les jeux
que le site web prend en charge actuellement, dont ceux de _StarCraft 2_.

Vous trouverez aussi la liste complète des _items_ de _StarCraft 2 Archipelago_ à la page
[_APSC2 Item Docs_](https://archipelago-sc2.github.io/content-docs/).
Notez que cette page contient diverses informations supplémentaires sur chacun des _items_.
Cependant, l'information présente dans cette dernière peut différer de celle du _datapackage_ d'Archipelago
puisqu'elle est générée, habituellement, à partir de la version en développement de _StarCraft 2 Archipelago_ qui
n'ont peut-être pas encore été inclus dans le site web d'Archipelago.

Pour ce qui concerne les _locations_, vous pouvez consulter tous les _locations_ associés à une mission dans votre
monde en plaçant votre curseur sur la case correspondante dans l'onglet _StarCraft 2 Launcher_ du client.

## Comment est-ce que je peux joindre un _MultiWorld_?

1. Exécuter `ArchipelagoStarcraft2Client.exe`.
   - Uniquement pour cette étape, les utilisateurs de macOS devraient plutôt suivre les instructions à la page
     ["Exécuter sous macOS"](#exécuter-sous-macos).
2. Entrer la commande `/connect [server ip]`.
   - Si le _MultiWorld_ est hébergé via un siteweb, l'IP du server devrait être indiqué dans le haut de la page de
     votre _room_.
3. Inscrivez le nom de joueur spécifié dans votre _yaml_ lorsque vous y êtes invité.
4. Si le serveur a un mot de passe, l'inscrire lorsque vous y êtes invité.
5. Une fois connecté, aller sur l'onglet _StarCraft 2 Launcher_ dans le client. Dans cet onglet, vous devriez trouver
   toutes les missions de votre monde. Les missions qui ne sont pas disponibles présentement auront leur texte dans une
   nuance de gris. Vous n'avez qu'à cliquer une des missions qui est disponible pour la commencer!

## _StarCraft 2_ ne démarre pas quand je tente de commencer une mission

Pour commencer, regarder le fichier _log_ pour trouver le problème (ce dernier devrait être dans
`[Archipelago Directory]/logs/SC2Client.txt`).
Si vous ne comprenez pas le problème avec le fichier _log_, visitez notre
[_Discord_](https://discord.com/invite/8Z65BR2) pour demander de l'aide dans le forum _tech-support_.
Dans votre message, veuillez inclure une description détaillée de ce qui ne marche pas et ajouter en pièce jointe le
fichier _log_.

## Mon profil de raccourcis clavier n'est pas disponibles quand je joue à _StarCraft 2 Archipelago_

Pour que votre profil de raccourcis clavier fonctionne dans Archipelago, vous devez copier votre fichier de raccourcis
qui se trouve dans `Documents/StarCraft II/Accounts/######/Hotkeys` vers `Documents/StarCraft II/Hotkeys`.
Si le dossier n'existe pas, créez-le.

Pour que _StarCraft 2 Archipelago_ utilise votre profil, suivez les étapes suivantes.
Lancez _StarCraft 2_ via l'application _Battle.net_.
Changez votre profil de raccourcis clavier pour le mode standard et acceptez, puis sélectionnez votre profil
personnalisé et acceptez.
Vous n'aurez besoin de faire ça qu'une seule fois.

## Exécuter sous macOS

Pour exécuter _StarCraft 2_ via Archipelago sous macOS, vous devez exécuter le client à partir de la source  
comme indiqué ici: [_macOS Guide_](/tutorial/Archipelago/mac/en).
Notez que pour lancer le client, vous devez exécuter la commande `python3 Starcraft2Client.py`.

## Exécuter sous Linux

Pour exécuter _StarCraft 2_ via Archipelago sous Linux, vous allez devoir installer le jeu avec _Wine_ et ensuite
exécuter le client d'Archipelago pour Linux.

Confirmez que vous avez installé _StarCraft 2_ via _Wine_ et que vous avez suivi les
[instructions d'installation](#comment-est-ce-que-j'installe-ce-randomizer?) pour ajouter les _Maps_ et les _Data
files_ nécessairent pour _StarCraft 2 Archipelago_ au bon endroit.
Vous n'avez pas besoin de copier les fichiers `.dll`.
Si vous avez des difficultés pour installer ou exécuter _StarCraft 2_ sous Linux, il est recommandé d'utiliser le
logiciel _Lutris_.

Copier ce qui suit dans un fichier avec l'extension `.sh`, en prenant soin de définir les variables **WINE** et
**SC2PATH** avec les bons chemins et de définir **PATH_TO_ARCHIPELAGO** avec le chemin vers le dossier qui contient le
_AppImage_ si ce dernier n'est pas dans le même dossier que ce script.

```sh
# Permet au client de savoir que SC2 est exécuté via Wine
export SC2PF=WineLinux
export PROTOCOL_BUFFERS_PYTHON_IMPLEMENTATION=python

# À_CHANGER Remplacer le chemin avec celui qui correspond à la version de Wine utilisé pour exécuter SC2
export WINE="/usr/bin/wine"

# À_CHANGER Remplacer le chemin par celui qui indique où StarCraft II est installé
export SC2PATH="/home/user/Games/starcraft-ii/drive_c/Program Files (x86)/StarCraft II/"

# À_CHANGER Indiquer le dossier qui contient l'AppImage d'Archipelago
PATH_TO_ARCHIPELAGO=

# Obtiens la dernière version de l'AppImage de Archipelago dans le dossier PATH_TO_ARCHIPELAGO.
# Si PATH_TO_ARCHIPELAGO n'est pas défini, la valeur par défaut est le dossier qui contient ce script.
ARCHIPELAGO="$(ls ${PATH_TO_ARCHIPELAGO:-$(dirname $0)}/Archipelago_*.AppImage | sort -r | head -1)"

# Lance le client de Archipelago
$ARCHIPELAGO "Starcraft 2 Client"
```

Pour une installation via Lutris, vous pouvez exécuter `lutris -l` pour obtenir l'identifiant numérique de votre
installation _StarCraft II_ et ensuite exécuter la commande suivante, en remplacant **${ID}** pour cet identifiant
numérique.

    lutris lutris:rungameid/${ID} --output-script sc2.sh

Cette commande va définir toutes les variables d'environnement nécessaires pour exécuter _StarCraft 2_ dans un script,
incluant le chemin vers l'exécutable _Wine_ que Lutris utilise.
Après ça, vous pouvez enlever la ligne qui permet de démarrer _Battle.Net_ et copier le code décrit plus haut dans le
script produit.
