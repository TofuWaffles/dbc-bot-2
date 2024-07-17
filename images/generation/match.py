import io
from typing import Optional, Tuple, Union
from PIL import Image, ImageDraw
from PIL.Image import Resampling
from pydantic import BaseModel
from .utils import Asset, Player, async_object
from .model import Background, Component


class RequestMatch(BaseModel):
    player1: Player
    player2: Player
    async def respond(self) -> Union[bytes, Exception]:
        image = await Match(self.player1, self.player2)
        if image.error:
            return image.error
        image.preset()
        image.build()
        return image.bytes()
      
class Match(async_object):
    asset = Asset()
    error: Optional[Exception] = None
    async def __init__(self, player1: Player, player2: Player):
      super().__init__()
      self.player1: Player = player1
      self.player2: Player = player2
      bg, self.error = self.asset.bg()
      fg, self.error = self.asset.fg()
      self.vs, self.error = self.asset.vs()
      self.vs_line, self.error = self.asset.vs_line()
      self.pvpd, self.error = self.asset.pvpd()
      self.pvpu, self.error = self.asset.pvpu()
      self.left_name, self.error = self.asset.left_name()
      self.right_name, self.error = self.asset.right_name()
      self.pi1, self.error = await self.asset.icon(self.player1.icon)
      self.pi2, self.error = await self.asset.icon(self.player2.icon)
      self.font, self.error = self.asset.font(30)
      self.bg = Image.new('RGBA', bg.size)
      self.bg.paste(im=fg, box=(0, 0))
      
    def write(self, img: Image.Image, text: str, position: Optional[Tuple[int, int]] = None, color: Tuple[int, int, int] = (0, 0, 0), align: str = 'left') -> None:
        """
        Write text on an image with specified alignment.

        Args:
            img (Image.Image): The image to write text on.
            text (str): The text to write.
            position (Optional[Tuple[int, int]]): The (x, y) position to start the text. Defaults to the center of the image.
            color (Tuple[int, int, int]): The color of the text in RGB.
            align (str): The alignment of the text. One of 'left', 'right', 'center'.
        """
        draw = ImageDraw.Draw(img)
        text_width = draw.textlength(text, font=self.font)

        if position is None:
            position = (img.width // 2, img.height // 2)

        x, y = position

        if align == 'center':
            x -= text_width // 2
        elif align == 'right':
            x -= text_width

        draw.text((x, y), text, font=self.font, fill=color)
    
    # def center(self) -> Tuple[int, int]:
    #   return self.bg.width // 2, self.bg.height // 2
    
    # def center_x(self) -> int:
    #   return self.bg.width // 2
    
    # def center_y(self) -> int:
    #   return self.bg.height // 2
    
    # def place_left(self, image: Image.Image, padding: int = 0) -> Tuple[int, int]:
    #   return padding, (self.bg.height - image.height) // 2
    
    # def place_right(self, image: Image.Image, padding: int = 0) -> Tuple[int, int]:
    #   return self.bg.width - image.width - padding, (self.bg.height - image.height) // 2
    
    # def place_top(self, image: Image.Image, padding: int = 0) -> Tuple[int, int]:
    #   return (self.bg.width - image.width) // 2, padding
    
    # def place_bottom(self, image: Image.Image, padding: int = 0) -> Tuple[int, int]:
    #   return (self.bg.width - image.width) // 2, self.bg.height - image.height - padding
    
    # def build(self) -> None:
    #   self.bg.paste(im=self.vs, box=self.center())
    #   self.bg.paste(im=self.pvpu, box=(0, 0))
    #   self.bg.paste(img=self.pvpd, box=(self.bg.width - self.pvpd.width, 0))
    #   padding = 30
    #   dimension = (100,100)
    #   self.pi1.resize(size=dimension, resample=Resampling.NEAREST)
    #   self.pi2.resize(size=dimension, resample=Resampling.NEAREST)
    #   self.pi1.position = self.place_left(self.pi1, padding)
    #   self.pi2.position = self.place_right(self.pi2, padding)
    #   self.bg.paste(im=self.pi1, box=self.pi1.position)
    #   self.bg.paste(im=self.pi2, box=self.pi2.position)
    #   self.bg.paste(im=self.left_name, box=(self.pi1.position[0]-(self.left_name.width-self.pi1.position)//2, self.pi1.position[1] + self.pi1.height+padding))
    #   self.bg.paste(im=self.right_name, box=(self.pi2.position[0]-(self.right_name.width-self.pi2.position)//2, self.pi2.position[1] + self.pi2.height+padding))
    def preset(self):
        background = Background(None, None, self.bg, "Match")
        ICON_SIZE = (200,200)
        vs = Component(img=self.vs.resize(size=(150, 150), resample=Resampling.NEAREST), pos=(0, 0), name="VS")
        vs.set_center_x(background.width)
        vs.set_center_y(background.height)
        vs_line = Component(img=self.vs_line.resize(size=(150, 150), resample=Resampling.NEAREST), pos=(0, 0), name="VS_Line")
        vs_line.set_center_x(background.width)
        icon1 = Component(img=self.pi1.resize(size=ICON_SIZE,resample=Resampling.NEAREST), pos=(50, 0), name="Icon1")
        icon1.set_center_y(background.height)
        icon2 = Component(img=self.pi2.resize(size=ICON_SIZE,resample=Resampling.NEAREST), pos=(0, 0), name="Icon2")
        icon2.set_x(background.width - icon2.width - icon1.x)
        icon2.set_center_y(background.height)
        pt = 10
        name1_layout = Component(img=self.left_name, pos=(0, 0), name="Name1")
        name1_layout.set_relative_center_x(icon1)
        name1_layout.set_y(icon1.y + icon1.height + pt)
        self.write(img=name1_layout.img, text=self.player1.discord_name, color=(255,255,255),align='center')
        name2_layout = Component(img=self.right_name, pos=(0, 0), name="Name2")
        name2_layout.set_relative_center_x(icon2)
        name2_layout.set_y(icon2.y + icon2.height + pt)
        self.write(img=name2_layout.img, text=self.player2.discord_name, color=(255,255,255),align='center')
        self.bg = background
        self.components = [vs, vs_line, icon1, icon2, name1_layout, name2_layout]
      
    def build(self) -> None:
        for component in self.components:
            self.bg.add_overlay(component)
        self.bg = self.bg.build()
    
    def bytes(self) -> bytes:
      output = io.BytesIO()
      self.bg.save(output, format="PNG")
      output.seek(0)
      return output.getvalue()
    
    
    

   