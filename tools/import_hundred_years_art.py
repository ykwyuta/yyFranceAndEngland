"""Normalize generated sprites without altering their artwork or alpha.

Run after placing the four source PNGs in assets/hundred-years/.
"""
from pathlib import Path
from PIL import Image, ImageOps, ImageDraw
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT / 'tools'))
from import_period_art import cells, bgra

DEST = ROOT / 'assets' / 'hundred-years'


def save_tile(tile, directory, name):
    tile.save(directory / (name + '.png'))
    (directory / (name + '.bgra')).write_bytes(bgra(tile))


def main():
    sheets = {}
    for name, columns, rows in [('units', 4, 4), ('buildings', 4, 3), ('heroes', 5, 5)]:
        source = Image.open(DEST / (name + '.png')).convert('RGBA')
        if source.getchannel('A').getextrema()[0] != 0:
            raise ValueError(f'{name}: source must have a genuinely transparent background')
        tiles = cells(DEST / (name + '.png'), columns, rows, 192)
        assert len(tiles) == columns * rows
        assert all(t.getchannel('A').getbbox() for t in tiles)
        sheets[name] = tiles
        (DEST / (name + '.bgra')).write_bytes(b''.join(bgra(t) for t in tiles))
    for name, index in [('worker', 0), ('infantry', 1), ('archer', 2), ('chariot', 3), ('cavalry', 4)]:
        tile = ImageOps.contain(sheets['units'][index], (160, 160), Image.Resampling.LANCZOS)
        save_tile(tile, ROOT / 'assets' / 'units', name)
    for name, index in [('capital', 0), ('town', 8), ('barracks', 4), ('farm', 5), ('fort', 6)]:
        save_tile(sheets['buildings'][index], ROOT / 'assets' / 'map', name)
    for name, index in [('mountain', 9), ('forest', 10), ('mine', 11)]:
        tile = sheets['buildings'][index].resize((160, 160), Image.Resampling.LANCZOS)
        save_tile(tile, ROOT / 'assets' / 'map', name)
    scene = Image.open(DEST / 'title.png').convert('RGBA').resize((768, 512), Image.Resampling.LANCZOS)
    (DEST / 'title.bgra').write_bytes(bgra(scene))
    scene.convert('RGB').save(ROOT / 'web' / 'key-art.jpg', quality=88)
    # Installation/executable icons use a period fortress instead of the inherited palace.
    icon = Image.new('RGBA', (512, 512), '#11141d')
    castle = sheets['buildings'][6].resize((448, 448), Image.Resampling.LANCZOS)
    icon.alpha_composite(castle, (32, 32))
    for filename, size in [('icon-512.png', 512), ('icon-192.png', 192), ('apple-touch-icon.png', 180)]:
        icon.resize((size, size), Image.Resampling.LANCZOS).save(ROOT / 'web' / filename)
    icon.save(ROOT / 'assets' / 'app_icon.ico', sizes=[(16,16), (32,32), (48,48), (128,128), (256,256)])
    # Review sheet contains only deterministic resizing and compositing of generated art.
    preview = Image.new('RGB', (8 * 144, 8 * 164), '#171a24')
    draw = ImageDraw.Draw(preview)
    all_tiles = [(name, i, tile) for name, tiles in sheets.items() for i, tile in enumerate(tiles)]
    for k, (name, i, tile) in enumerate(all_tiles):
        x, y = k % 8 * 144, k // 8 * 164
        draw.text((x + 8, y + 4), f'{name} {i:02}', fill='#ccb781')
        small = tile.resize((140,140), Image.Resampling.LANCZOS)
        preview.paste(small, (x+2,y+22), small)
    preview.save(DEST / 'catalog.jpg', quality=92)


if __name__ == '__main__':
    main()
