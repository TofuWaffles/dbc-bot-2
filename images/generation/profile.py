import io
from PIL import Image
from pydantic import BaseModel
from .utils import Asset, Player

class RequestProfile(BaseModel):
    player: Player
    def respond(self) -> bytes:
        image = Profile(self.player)
        return image.build()
      
class Profile:
    asset = Asset()
    def __init__(self, player: Player) -> None:
      self.player: Player = player
      self.bg = self.asset.bg()
      self.fg = self.asset.fg()
      self.bg = Image.new('RGBA', self.bg.size)
      self.bg.paste(im=self.fg, box=(0, 0))
      
    def build(self) -> bytes:
      output = io.BytesIO()
      self.bg.save(output, format="PNG")
      output.seek(0)
      return output.getvalue()
    
    
    

   