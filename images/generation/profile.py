<<<<<<< HEAD
from typing import Union
from pydantic import BaseModel
from .model import Background, Component, BaseImage, Player
import base64 

class RequestProfile(BaseModel):
    player: Player
    async def respond(self) -> Union[str, Exception]:
        image = await Profile(self.player)
        if image.error:
            return image.error
        image.preset()
        image.build()
        encode = base64.b64encode(image.bytes()).decode("utf-8")
        return encode
      
class Profile(BaseImage):
    async def __init__(self, player: Player) -> None:
        await super().__init__()
        self.player: Player = player
        self.bg = self.asset.get_image("Player_clean.png")
      
    def preset(self) -> None:
        pass
    
    
    

=======
from typing import Union
from pydantic import BaseModel
from .model import Background, Component, BaseImage, Player
import base64 

class RequestProfile(BaseModel):
    player: Player
    async def respond(self) -> Union[str, Exception]:
        image = await Profile(self.player)
        if image.error:
            return image.error
        image.preset()
        image.build()
        encode = base64.b64encode(image.bytes()).decode("utf-8")
        return encode
      
class Profile(BaseImage):
    async def __init__(self, player: Player) -> None:
        await super().__init__()
        self.player: Player = player
        self.bg = self.asset.get_image("Player_clean.png")
      
    def preset(self) -> None:
        pass
    
    
    

>>>>>>> bdb70236c68496a534164a78cf20d5436eba400c
   