# _StarCraft 2_

## Quel est l'effet de la _randomization_ sur ce jeu ?

### _Items_ et _locations_

Les éléments qui suivent sont les _items_ qui sont _randomized_ et qui doivent être débloqués pour être utilisés dans
le jeu:

1. La capacité de produire des unités, excepté les drones/probes/scv.
2. Des améliorations spécifiques à certaines unités incluant quelques combinaisons qui ne sont pas disponibles dans les
   campagnes génériques, comme le fait d'avoir les deux types d'évolution en même temps pour une unité _Zerg_ et toutes
   les améliorations de la _Spear of Adun_ simultanément pour les _Protoss_.
3. L'accès aux améliorations génériques des unités, e.g. les améliorations d'attaque et d'armure.
4. D'autres améliorations diverses telles que les améliorations de laboratoire et les mercenaires pour les _Terran_,
   les niveaux et les améliorations de Kerrigan pour les _Zerg_, et les améliorations de la _Spear of Adun_ pour les
   _Protoss_.
5. Avoir des _minerals_, du _vespene gas_, et du _supply_ au début de chaque mission.

Les _items_ sont trouvés en accomplissant du progrès dans les catégories suivantes:

- Terminer des missions
- Réussir des objectifs supplémentaires (e.g., récolter le matériel pour les recherches dans _Wings of Liberty_)
- Atteindre des étapes importantes dans la mission, e.g. réussir des sous-objectifs
- Réussir des défis basés sur les succès du jeu de base, e.g. éliminer tous les _Zerg_ dans la mission
  _Devil's Playground_

Dans la nomenclature d'Archipelago, il s'agit des _locations_ où l'on peut trouver des _items_.
Pour chaque _location_, incluant le fait de terminer une mission, il y a des règles qui définissent les _items_
nécessaires pour y accéder.
Ces règles ont été conçues en assumant que _StarCraft 2_ est joué à la difficulté _Brutal_.
Étant donné que chaque _location_ a ses propres règles, il est possible qu'un _item_ nécessaire à la progression se
trouve dans une mission dont vous ne pouvez pas atteindre toutes les _locations_ ou que vous ne pouvez pas terminer.
Cependant, il est toujours nécessaire de terminer une mission pour pouvoir accéder à de nouvelles missions.

Ces catégories, outre la première, peuvent être désactivées dans les options du jeu.
Par exemple, vous pouvez désactiver le fait d'obtenir des _items_ lorsque des étapes importantes d'une mission sont
accomplies.

Quand vous recevez un _item_, il devient immédiatement disponible, même pendant une mission, et vous serez avertis via
la boîte de texte situé dans le coin en haut à droite de _StarCraft 2_.
L'acquisition d'un _item_ est aussi indiquée dans le client d'Archipelago.

### _Mission order_

Les missions et l'ordre dans lequel elles doivent être complétées, dénoté _mission order_, peuvent également être
_randomized_.
Les quatre campagnes de _StarCraft 2_ peuvent être utilisées pour remplir le _mission order_.
Notez que les missions d'évolution de _Heart of the Swarm_ ne sont pas incluses dans le _randomizer_.
Par défaut, le _mission order_ suit la structure des campagnes sélectionnées, mais plusieurs autres options sont
disponibles, comme _blitz_, _grid_, etc.

Les missions peuvent être lancées par le client _StarCraft 2 Archipelago_, via l'interface graphique de l'onglet
_StarCraft 2 Launcher_.
Les segments qui se passent sur l'_Hyperion_, un Léviathan et la _Spear of Adun_ ne sont pas inclus.
De plus, les points de progression, tels que les crédits ou la Solarite, ne sont pas utilisés dans _StarCraft 2
Archipelago_.
Les missions accessibles ont leur nom en bleu, tandis que celles où toutes les _locations_ ont été collectées
apparaissent en blanc.
En plaçant votre souris sur une mission, les _locations_ non collectées s’affichent, classées par catégorie.
Les missions qui ne sont pas accessibles ont leur nom en gris et leurs prérequis seront également affichés à cet endroit.

## Quel est le but de ce jeu quand il est _randomized_?

Le but est de réussir la mission finale du _mission order_ (e.g. _blitz_, _grid_, etc.).
Le fichier de configuration yaml permet de spécifier le _mission order_, quelle combinaison des quatre campagnes de
_StarCraft 2_ peuvent être utilisée et comment les missions sont distribuées dans le _mission order_.
Étant donné que les deux premières options déterminent le nombre de missions dans un monde de _StarCraft 2_, elles
peuvent être utilisées pour moduler le temps nécessaire pour terminer le monde.

## Quelles sont les modifications non aléatoires comparativement à la version de base de _StarCraft 2_

1. Certaines des missions ont plus de _vespene geysers_ pour permettre l'utilisation d'une plus grande variété d'unités.
2. Plusieurs unités et améliorations ont été ajoutées sous la forme d*items*.
   Ils proviennent de la version _co-op_, _melee_, des autres campagnes, d'expansions ultérieures, de _Brood War_, ou de
   l'imagination des développeurs de _StarCraft 2 Archipelago_.
3. Les structures de production, e.g. _Factory_, _Starport_, _Robotics Facility_, and _Stargate_, n'ont plus
   d'exigences technologiques.
4. Les missions avec la race _Zerg_ ont été modifiées pour que les joueurs débuttent avec un _Lair_ lorsqu'elles
   commençaient avec une _Hatchery_.
5. Les désavantages des améliorations ont été enlevés, e.g. _automated refinery_ qui coûte plus cher ou les _tech
   reactors_ qui prennent plus de temps à construire.
6. La collision des unités dans les couloirs de la mission _Enemy Within_ a été ajustée pour permettre des unités
   plus larges de les traverser sans être coincés dans des endroits étranges.
7. Plusieurs _bugs_ du jeu original ont été corrigés.

## Quels sont les _items_ qui peuvent être dans le monde d'un autre joueur?

Par défaut, tous les _items_ de _StarCraft 2 Archipelago_ (voir la section précédente) peuvent être dans le monde d'un
autre joueur.
Consulter [_Advanced YAML Guide_](/tutorial/Archipelago/advanced_settings/en) pour savoir comment
changer ça.

## Commandes du client qui sont uniques à ce jeu

Les commandes qui suivent sont seulement disponibles uniquement pour le client de _StarCraft 2 Archipelago_.
Vous pouvez les afficher en utilisant la commande `/help` dans le client de _StarCraft 2 Archipelago_.
Toutes ces commandes affectent seulement le client où elles sont utilisées.

- `/download_data` Télécharge les versions les plus récentes des fichiers pour jouer à _StarCraft 2 Archipelago_.
  Les fichiers existants vont être écrasés.
- `/difficulty [difficulty]` Remplace la difficulté choisie pour le monde.
  - Les options sont _casual_, _normal_, _hard_, et _brutal_.
- `/game_speed [game_speed]` Remplace la vitesse du jeu pour le monde.
  - Les options sont _default_, _slower_, _slow_, _normal_, _fast_, and _faster_.
- `/color [faction] [color]` Remplace la couleur d'une des _factions_ qui est jouable.
  - Si la commande est lancée sans option, la liste des _factions_ et des couleurs disponibles sera affichée.
- `/option [option_name] [option_value]` Permet de changer un option normalement définit dans le _yaml_.
  _ Si la commande est lancée sans option, la liste des options qui sont modifiables va être affichée.
  _ Les options qui peuvent être changées avec cette commande incluent sauter les cinématiques automatiquement, la
  présence de Kerrigan dans les missions, la disponibilité de la _Spear of Adun_, la quantité de ressources
  supplémentaires données au début des missions, la capacité de contrôler les alliées IA, etc.
- `/disable_mission_check` Désactive les requit pour lancer les missions.
  Cette option a pour but de permettre de jouer en mode coopératif en permettant à un joueur de jouer à la prochaine
  mission de la chaîne qu'un autre joueur est en train d'entamer.
- `/set_path [path]` Permet de définir manuellement où _StarCraft 2_ est installé ce qui est pertinent seulement si la
  détection automatique de cette dernière échoue.

Notez que le comportement de la commande `/received` a été modifié dans le client _StarCraft 2_.
Dans le client _Common_ d'Archipelago, elle renvoie la liste des _items_ reçus dans l'ordre inverse de leur réception.
Dans le client de _StarCraft 2_, la liste est divisée par races (i.e., _Any_, _Protoss_, _Terran_, et _Zerg_).
De plus, les améliorations sont regroupées sous leurs unités/bâtiments correspondants.
Un paramètre de filtrage peut aussi être fourni, e.g., `/received Thor`, pour limiter le nombre d'_items_ affichés.
Tous les _items_ dont le nom, la race ou le nom de groupe contient le paramètre fourni seront affichés.

## Particularités dans un multiworld

### _Collect on goal completion_

L'une des options par défaut des _multiworlds_ est qu'une fois qu'un monde a atteint son objectif final, il collecte
tous ses _items_, incluant ceux dans les autres mondes.
Si vous ne souhaitez pas que cela se produise, vous devez demander à la personne générant le _multiworld_ de changer
l'option _Collect Permission_.
Si la génération n'est pas effectuée via le site web, la personne qui effectue la génération doit modifier l'option
`collect_mode` dans son fichier _host.yaml_ avant la génération.
Si le _multiworld_ a déjà été généré, l'hôte peut utiliser la commande `/option collect_mode [valeur]` pour modifier
cette option.

## Problèmes connus

- _StarCraft 2 Archipelago_ ne supporte pas le chargement d'une sauvegarde.
  Pour cette raison, il est recommandé de jouer à un niveau de difficulté inférieur à celui avec lequel vous êtes
  normalement à l'aise.
- _StarCraft 2 Archipelago_ ne supporte pas le redémarrage d'une mission depuis le menu de _StarCraft 2_.
  Pour redémarrer une mission, utilisez le client de _StarCraft 2 Archipelago_.
- Un rapport d'erreur est souvent généré lorsqu'une mission est fermée.
  Cela n'affecte pas le jeu et peut être ignoré.
- Actuellement, le client de _StarCraft 2_ utilise la _location_ associée à la victoire d'une mission pour déterminer
  si celle-ci a été complétée.
  En conséquence, la fonctionnalité _collect_ d'_Archipelago_ peut rendre accessible des missions connectées à une
  mission que vous n'avez pas terminée.
