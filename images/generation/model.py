from PIL import Image, ImageOps, ImageChops
from typing import Optional, List

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

    def overlay(self, top: Image.Image) -> None:
        # Calculate overlay bounds
        box = (
            max(self.x, 0),
            max(self.y, 0),
            min(self.x + self.width, top.width),
            min(self.y + self.height, top.height)
        )
        region = top.crop((box[0] - self.x, box[1] - self.y, box[2] - self.x, box[3] - self.y))
        self.img.paste(region, (box[0], box[1]), region)

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
            final_image.paste(overlay.img, (overlay.x, overlay.y))
        return final_image