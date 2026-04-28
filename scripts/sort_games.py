# Reorders the games.json file alphabetically by name.

from utils import read_games, write_games

def sort_games():
    (json_path, games) = read_games()

    sort = sorted(games, key=lambda game: game['name'])

    write_games(json_path, sort)

if __name__ == '__main__':
    sort_games()