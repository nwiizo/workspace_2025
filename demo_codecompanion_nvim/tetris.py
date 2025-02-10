
import pygame
import sys
import json

# 初期化
pygame.init()

# 画面サイズ
screen_width = 800
screen_height = 600
screen = pygame.display.set_mode((screen_width, screen_height))
pygame.display.set_caption("Tetris")

# 色の定義
black = (0, 0, 0)
white = (255, 255, 255)

def load_block_data(file_path):
    with open(file_path, 'r') as file:
        block_data = json.load(file)
    return block_data

# ブロックデータの読み込み
block_data = load_block_data('blocks.json')


# ゲームループ

def draw_block(block, x, y, color):
    block_size = 30  # ブロックのサイズ
    for row in range(len(block)):
        for col in range(len(block[row])):
            if block[row][col] == 1:
                pygame.draw.rect(screen, color, pygame.Rect(x + col * block_size, y + row * block_size, block_size, block_size))

# ゲームループの中でブロックを描画
def game_loop():
      
    while True:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                pygame.quit()
                sys.exit()

        screen.fill(black)
        
        # ブロックの描画例
        draw_block(block_data['I'], 100, 100, white)
        
        pygame.display.flip()
      

if __name__ == "__main__":
    game_loop()
      ame_loop()
      