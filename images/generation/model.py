from typing import Optional, List
from pydantic import BaseModel
from pathlib import Path
from typing import Optional, Tuple
from PIL import Image, ImageFont, ImageDraw
import httpx
import io
class Player(BaseModel):
    discord_id: str
    discord_name: str
    player_tag: str
    player_name: str
    player_name: str
    icon: int  

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

class Component:
  
    def __init__(self, img: Image.Image, pos: tuple[int, int], name: str = "untitled"):
        """
        A component is a part of the image that can be moved around and resized.
        Args:
            img (Image.Image): The image to be used as a component.
            pos (tuple): The position of the component on the image.
            name (str): The name of the component.
        """
        self.img: Image.Image = img.copy()
        self.x: int = pos[0]
        self.y: int = pos[1]
        self.name: str = name

    @property
    def width(self) -> int:
        return self.img.width

    @property
    def height(self) -> int:
        return self.img.height

    def set_x(self, x: int) -> None:
        self.x = x

    def set_y(self, y: int) -> None:
        self.y = y

    def set_center_x(self, parent_width: int) -> None:
        self.x = (parent_width - self.img.width) // 2

    def set_center_y(self, parent_height: int) -> None:
        self.y = (parent_height - self.img.height) // 2

    def set_relative_center_x(self, dependent: 'Component') -> None:
        self.set_center_x(2 * dependent.x + dependent.width)

    def set_relative_center_y(self, dependent: 'Component') -> None:
        self.set_center_y(2 * dependent.y + dependent.height)

    def set_dimensions(self, width: int, height: int) -> None:
        self.img = self.img.resize((width, height), Image.LANCZOS)

    def get_center_x(self, base_width: int) -> int:
        return (base_width - self.img.width) // 2

    def get_center_y(self, base_height: int) -> int:
        return (base_height - self.img.height) // 2

    def overlay(self, bottom: Image.Image, position: tuple[int, int]) -> None:
        x, y = position
        bottom.paste(self.img, (x, y))
        
    def has_transparency(self) -> bool:
        if self.img.info.get("transparency", None) is not None:
            return True
        if self.img.mode == "P":
            transparent = self.img.info.get("transparency", -1)
            for _, index in self.img.getcolors():
                if index == transparent:
                    return True
        elif self.img.mode == "RGBA":
            extrema = self.img.getextrema()
            if extrema[3][0] < 255:
                return True
        return False

class Background:
    def __init__(self, width: Optional[int] = None, height: Optional[int] = None, bg: Image.Image = None, name: str = "untitled"):
        self.bg: Image.Image = Image.new('RGBA', (width or 1920, height or 1080), (0, 0, 0, 0)) if bg is None else bg
        self.width: int = width or self.bg.width
        self.height: int = height or self.bg.height
        self.name: str = name
        self.overlays: List[Component] = []

    def add_overlay(self, overlay: Component) -> None:
        self.overlays.append(overlay)

    def build(self) -> Image.Image:
        final_image: Image.Image = self.bg.copy()
        for overlay in self.overlays:
            top = overlay.img
            if overlay.has_transparency():
                final_image.paste(top, (overlay.x, overlay.y), top)
            else:
                final_image.paste(top, (overlay.x, overlay.y))
        return final_image
    

class BaseImage:
    """Inheriting this class allows you to define an async __init__.

    So you can create objects by doing something like `await MyClass(params)`
    """
    asset: Asset = Asset()
    error: Optional[Exception] = None
    components: List[Component] = []
    async def __new__(cls, *a, **kw):
        instance = super().__new__(cls)
        await instance.__init__(*a, **kw)
        return instance
    
    async def __init__(self): 
        self.font, self.error = self.asset.font(30)
        
    
    def get_text_size(self, text: str) -> Tuple[int, int]:
        """
        Get the width and height of the text.

        Args:
            text (str): The text to measure.

        Returns:
            Tuple[int, int]: The width and height of the text.
        """
        draw = ImageDraw.Draw(self.bg)
        bbox = draw.textbbox((0, 0), text, font=self.font)
        text_width = bbox[2] - bbox[0]
        text_height = bbox[3] - bbox[1]
        return text_width, text_height
    
    def write(self, text: str, textbox_pos: Tuple[Tuple[int, int], Tuple[int, int]], align: str = "left", color: Tuple[int, int, int] = (0, 0, 0)) -> None:
        """
        Write text on an image with specified alignment.

        Args:
            text (str): The text to write.
            textbox_pos (Tuple[Tuple[int, int], Tuple[int, int]]): The position of the text box in the format ((x1, y1), (x2, y2)).
            align (str): The horizontal alignment ('left', 'center', 'right').
            color (Tuple[int, int, int]): The color of the text in RGB.
        """
        draw = ImageDraw.Draw(self.bg)
        text_width, text_height = self.get_text_size(text)
        
        x1, y1 = textbox_pos[0]
        x2, y2 = textbox_pos[1]

        # Calculate horizontal position
        match align:
            case "right":
                x = x2 - text_width
            case "center":
                x = x1 + (x2 - x1 - text_width) // 2
            case _:
                x = x1
        
        # Calculate vertical position for centering
        y = y1 + (y2 - y1 - text_height) // 2
        
        # Draw the text
        draw.text((x, y), text, font=self.font, fill=color)
        
    def build(self) -> None:
        for component in self.components:
            self.bg.add_overlay(component)
        self.bg = self.bg.build()
    
    def bytes(self) -> bytes:
      output = io.BytesIO()
      self.bg.save(output, format="PNG")
      output.seek(0)
      return output.getvalue()
    
