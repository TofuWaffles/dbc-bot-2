import io
from pathlib import Path
from typing import Optional, Tuple
from PIL import Image, ImageFont
import httpx
import requests
from pydantic import BaseModel

class Player(BaseModel):
    discord_id: str
    discord_name: str
    player_tag: str
    player_name: str
    player_name: str
    icon: int   

class async_object:
    """Inheriting this class allows you to define an async __init__.

    So you can create objects by doing something like `await MyClass(params)`
    """
    async def __new__(cls, *a, **kw):
        instance = super().__new__(cls)
        await instance.__init__(*a, **kw)
        return instance

    async def __init__(self):
        pass

class Asset:
    def __init__(self):
        current_dir = Path(__file__).resolve().parent
        parent_dir = current_dir.parent
        self.assets_dir = parent_dir / 'assets'

    def get_image(self, filename: str) -> Tuple[Optional[Image.Image], Optional[Exception]]:
        try:
            img_path = self.assets_dir / filename
            return Image.open(img_path), None
        except FileNotFoundError as e:
            return None, e
        except Exception as e:
            return None, e
        

    def bg(self) -> Tuple[Optional[Image.Image], Optional[Exception]]:
        return self.get_image('Background.png')

    def fg(self) -> Tuple[Optional[Image.Image], Optional[Exception]]:
        return self.get_image('Foreground.png')

    def vs(self) -> Tuple[Optional[Image.Image], Optional[Exception]]:
        return self.get_image('Vs_sign.png')
    
    def vs_line(self) -> Tuple[Optional[Image.Image], Optional[Exception]]:
        return self.get_image('Vs_line.png')

    def pvpd(self) -> Tuple[Optional[Image.Image], Optional[Exception]]:
        return self.get_image('PVP_D.png')

    def pvpu(self) -> Tuple[Optional[Image.Image], Optional[Exception]]:
        return self.get_image('PVP_U.png')

    def left_name(self) -> Tuple[Optional[Image.Image], Optional[Exception]]:
        return self.get_image('PlayerName_Left.png')

    def right_name(self) -> Tuple[Optional[Image.Image], Optional[Exception]]:
        return self.get_image('PlayerName_Right.png')

    def font(self, size: int) -> Tuple[Optional[ImageFont.ImageFont], Optional[Exception]]:
        try:
            font_path = self.assets_dir / 'lilitaone-regular.ttf'
            return ImageFont.truetype(str(font_path), size=size), None
        except FileNotFoundError as e:
            return None, e
        except Exception as e:
            return None, e

    async def icon(self, id: str) -> Tuple[Optional[Image.Image], Optional[Exception]]:
        url = f"https://cdn-old.brawlify.com/profile/{id}.png"
        # response = requests.get(url)
        # bytes_ = response.content
        # image = Image.open(io.BytesIO(bytes_))
        # return image, None
        try:
            async with httpx.AsyncClient() as client:
                response = await client.get(url)
                response.raise_for_status()
                bytes_ = response.content 
                image = Image.open(io.BytesIO(bytes_)) 
                return image, None
        except httpx.HTTPStatusError as e:
            return None, e
        except Exception as e:
            return None, e
        finally:
            await client.aclose()
