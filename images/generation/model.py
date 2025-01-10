from asyncio.log import logger
import io
from pathlib import Path
from typing import List, Optional, Tuple, Union
from PIL.Image import Resampling
import httpx
from PIL import Image, ImageDraw, ImageFont

from pydantic import BaseModel


class Mode:
    _icon = {
        "basketBrawl": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC81dWlqV0dnUXFCcHBRV2lCMjY3NS5wbmcifQ:supercell:b0UVYB2iojQNxd8cgfsoq5FdwRFUPr4uG_rPSTNBoUk?width=2400",
        "bigGame": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9hNXc5NHFMM3NoSml6VnJvd291dC5wbmcifQ:supercell:kegT25IKJXUqJ_zIWQHCjl6fTzyLwoZMjfSLJQZv8L4?width=2400",
        "bossFight": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9oNVM4TWhzOEZjQXRnM3hRR0pHUC5wbmcifQ:supercell:-ZnZmzd0cEiV8sUoRpms1E18dNy0idZ54bcbWG81FhI?width=2400",
        "botdrop": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9rZVJUOHdqalZ4VUc3VUtlZjhvai5wbmcifQ:supercell:jIhfS7rjmt4uUkda7OQZNNbOAqVxpMWGW_0HdyS-a0A?width=2400",
        "bounty": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC82QUR6RUR0ZGJVSzYzZ3NzV3p2ci5wbmcifQ:supercell:hvUWH6mD3onZQhEnVHEFiR0fJoGjtZyxjye3pRTYE3w?width=2400",
        "brawlBall": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9SM1IyckRRaHBaREhMa0NCejlBUC5wbmcifQ:supercell:WkDBho-g-Ebpc8OT3q-1Y7SSmio4BXPXHTUDI8Z6SKU?width=2400",
        "brawlBall5v5": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9SM1IyckRRaHBaREhMa0NCejlBUC5wbmcifQ:supercell:WkDBho-g-Ebpc8OT3q-1Y7SSmio4BXPXHTUDI8Z6SKU?width=2400",
        "drumRoll": "https://static.wikia.nocookie.net/brawlstars/images/3/3e/Drum_Roll.png/",
        "duels": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9HMnhNUTljaXJRWFJUMlFpQ1VKbS5wbmcifQ:supercell:fZN_Hrzu2rqpc6Yrf0nv3X_FKFxq145Iw6faw7SAeV8?width=2400",
        "duoShowdown": "https://cdn-old.brawlify.com/gamemode/Duo-Showdown.png",
        "gemGrab": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9mb2dXOEpvbkVpV2YzTFl0b0NNaS5wbmcifQ:supercell:3MGSQtlWFyaOAbGHmoBpr6FVdLNzQH8fbAOIUiEKODk?width=2400",
        "gemGrab5v5": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9mb2dXOEpvbkVpV2YzTFl0b0NNaS5wbmcifQ:supercell:3MGSQtlWFyaOAbGHmoBpr6FVdLNzQH8fbAOIUiEKODk?width=2400",
        "godzillaCitySmash": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9NdGV2ZUhhcUg3UExrZnR3OXU1Ui5wbmcifQ:supercell:drSUFUPZuz30RjD8NuqcDmIkT_07kW-806cGDd6vjC8?width=2400",
        "heist": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC93MmtCcnlHZk05eHdYbWtCZWpCaC5wbmcifQ:supercell:62YMWTV9LI8syf1HAJnKJTMkUEZR1-yXNqrxVHTHrB4?width=2400",
        "holdTheTrophy": "https://cdn-old.brawlify.com/gamemode/Hold-The-Trophy.png",
        "hotZone": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9BVXdaY01mYkJvRWN3Sk5rdUR0ZC5wbmcifQ:supercell:iZg6LAmyaKSEVwstxj2JIAlgOZgpSFMIa0yVmnaoP7Q?width=2400",
        "hunters": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC80RVp2MWExbWl5R0F2bjNrREFkby5wbmcifQ:supercell:RFw0aCCSi-Ric4jKrRKzyB4NaGAFWHvSQc-LltPtRVE?width=2400",
        "jellyfishing": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9xUVBXcjE1bWplQmRZeWdvdkNlYi5wbmcifQ:supercell:lzbVb2WRzJeCTOPRnagySsMTlHkSvQU0eSkb7sX2WX0?width=2400",
        "knockout": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9nOVFZVnp1U1Z4cTM3MlJVWEx0ZS5wbmcifQ:supercell:JeQOURIdLohQGssx9Gm3eWexJD-LMeB47MZYqqp039Q?width=2400",
        "knockout5v5": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9nOVFZVnp1U1Z4cTM3MlJVWEx0ZS5wbmcifQ:supercell:JeQOURIdLohQGssx9Gm3eWexJD-LMeB47MZYqqp039Q?width=2400",
        "lastStand": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9kb2V0Y2dBVHY2bWdRRGpWczdINy5wbmcifQ:supercell:Yjx2YMYpXVZyyOXgh0fVh-embjX21h4nSYGZwOTPOX0?width=2400",
        "lonestar": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9LS2pKbjZwbVI1ekxMelRQMUtBVS5wbmcifQ:supercell:igMxWFKKr71Ib30FbRxR9X4KbesU8s_LD87SptdNOOs?width=2400",
        "mirrorMatch": "https://cdn-old.brawlify.com/gamemode/Mirror-Match.png",
        "paintBrawl": "https://static.wikia.nocookie.net/brawlstars/images/2/2d/Paint_Brawl.png",
        "payload": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9NR3RIV0pnNFJTb0NRZjJRYmlkOC5wbmcifQ:supercell:LQyf1RbrlYUuMCmCQVfhilgxb8COfr_ySz4h3-VWTiI?width=2400",
        "presentPlunder": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9tQ3FoNzJ4aGVYWHIxQlZNUThvby5wbmcifQ:supercell:JpvyeFrl_1oiiLJNb6Qxc6NU72QpQWoMt9wyESriNXY?width=2400",
        "pumpkinPlunder": "https://static.wikia.nocookie.net/brawlstars/images/2/2b/Pumpkin_Plunder.png/",
        "roboRumble": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9vNUZVb3BQTHhOTVN0bkJibTJqUC5wbmcifQ:supercell:QZDhPMLHPbDjLkLc8rFAx281OWxl8dRVrl01B79C0Tc?width=2400",
        "siege": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9IdHZNc2lERE5zYUhSWm43MVA1ei5wbmcifQ:supercell:Gj3FsZnr_yv6_gOfl__nfzLYHS9X1BkyaGzY1fVe51E?width=2400",
        "soloShowdown": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9nbkxKdVIxRHlZTE1qUlMzV2pMTC5wbmcifQ:supercell:OfOmUHA0JeavDf4uX8SuyvQBmOq0AwN8aKuXPiM946Q?width=2400",
        "superCityRampage": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9TZGNkVUNRbldrc3ZaNWJoU0s1cS5wbmcifQ:supercell:CgmWVJFvWm7TCkdvJpGyH2MbDHl9FaxvXg9sL591LQ0?width=2400",
        "takedown": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9xSmNQYTlITE5jV1l6NTRob3NwdC5wbmcifQ:supercell:i0UcwwmUSutjuFyddNEkT_D1YOwXfsAtqeILkzy95xw?width=2400",
        "trioShowdown": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9lRWFFOFdwUXRoWGZLaUdGaFd6Qi5wbmcifQ:supercell:sH0_yvkm8pQIXuf8pu-z2NVT2HO10-VuOqdPGiL9DOg?width=2400",
        "volleyBrawl": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC9GMnNQRVh0M0NmZnlwWnBBRmZvZy5wbmcifQ:supercell:aPmpvqJ_kWVHsjcJUME6ZVPmI_RYEggMSxywBMTuhFY?width=2400",
        "wipeout": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC8zYkd1MWg2cDNuNjZmeDlyZkJ4Ri5wbmcifQ:supercell:ayUKS371774s5SIUky4GvlgHR-lUcbkt9F1j31SZR44?width=2400",
        "wipeout5v5": "https://cdn-assets-eu.frontify.com/s3/frontify-enterprise-files-eu/eyJwYXRoIjoic3VwZXJjZWxsXC9maWxlXC8zYkd1MWg2cDNuNjZmeDlyZkJ4Ri5wbmcifQ:supercell:ayUKS371774s5SIUky4GvlgHR-lUcbkt9F1j31SZR44?width=2400",
    }
    _name = {
        "basketBrawl": "Basket Brawl",
        "bigGame": "Big Game",
        "bossFight": "Boss Fight",
        "botdrop": "Bot Drop",
        "bounty": "Bounty",
        "brawlBall": "Brawl Ball",
        "brawlBall5v5": "Brawl Ball 5v5",
        "drumRoll": "Drum Roll",
        "duels": "Duels",
        "duoShowdown": "Duo Showdown",
        "gemGrab": "Gem Grab",
        "gemGrab5v5": "Gem Grab 5v5",
        "godzillaCitySmash": "Godzilla City Smash",
        "heist": "Heist",
        "holdTheTrophy": "Hold The Trophy",
        "hotZone": "Hot Zone",
        "hunters": "Hunters",
        "jellyfishing": "Jellyfishing",
        "knockout": "Knockout",
        "knockout5v5": "Knockout 5v5",
        "lastStand": "Last Stand",
        "lonestar": "Lonestar",
        "mirrorMatch": "Mirror Match",
        "paintBrawl": "Paint Brawl",
        "payload": "Payload",
        "presentPlunder": "Present Plunder",
        "pumpkinPlunder": "Pumpkin Plunder",
        "roboRumble": "Robo Rumble",
        "siege": "Siege",
        "soloShowdown": "Solo Showdown",
        "superCityRampage": "Super City Rampage",
        "takedown": "Takedown",
        "trioShowdown": "Trio Showdown",
        "volleyBrawl": "Volley Brawl",
        "wipeout": "Wipeout",
        "wipeout5v5": "Wipeout 5v5",
    }

    @staticmethod
    def name(mode: str) -> str:
        return Mode._name.get(mode, "Unknown")

    @staticmethod
    def icon(mode: str) -> str:
        return Mode._icon.get(mode)


class Player(BaseModel):
    discord_id: str
    discord_name: str
    player_tag: str
    player_name: str
    icon: int


class Brawler(BaseModel):
    name: str
    id: int


class ExtendedPlayer(Player):
    discord_id: str
    discord_name: str
    player_tag: str
    player_name: str
    icon: int
    brawler: Brawler


class Asset:
    def __init__(self):
        current_dir = Path(__file__).resolve().parent
        parent_dir = current_dir.parent
        self.assets_dir = parent_dir / "assets"

    def get_image(
        self, filename: str
    ) -> Tuple[Optional[Image.Image], Optional[Exception]]:
        try:
            img_path = self.assets_dir / filename
            return Image.open(img_path), None
        except FileNotFoundError as e:
            return None, e
        except Exception as e:
            return None, e

    def font(
        self, size: int
    ) -> Tuple[Optional[ImageFont.ImageFont], Optional[Exception]]:
        try:
            font_path = self.assets_dir / "lilitaone-regular.ttf"
            return ImageFont.truetype(str(font_path), size=size), None
        except FileNotFoundError as e:
            return None, e
        except Exception as e:
            return None, e

    @staticmethod
    async def icon(id: int | str) -> Tuple[Optional[Image.Image], Optional[Exception]]:
        url = f"https://cdn.brawlify.com/profile-icons/regular/{id}.png"
        try:
            async with httpx.AsyncClient() as client:
                response = await client.get(url)
                response.raise_for_status()
                bytes_ = response.content
                image = Image.open(io.BytesIO(bytes_))
                if image is None:
                    return Asset.get_image("28000000.png"), None
                return image, None
        except (httpx.HTTPStatusError, OSError) as e:
            logger.error(f"An error occurred while fetching the player icon: {e}")
            return Asset.get_image("28000000.png"), None
        except Exception as e:
            logger.error(f"An error occurred while fetching the player icon: {e}")
            return Asset.get_image("28000000.png"), None

    @staticmethod
    async def get_mode_icon(
        mode: str,
    ) -> Tuple[Optional[Image.Image], Optional[Exception]]:
        try:
            url = Mode.icon(mode)
            async with httpx.AsyncClient() as client:
                response = await client.get(url)
                response.raise_for_status()
                bytes_ = response.content
                image = Image.open(io.BytesIO(bytes_)).resize(size=(735, 735), resample=Resampling.NEAREST)
                
                return image, None
        except KeyError as e:
            return None, e
        except httpx.HTTPStatusError as e:
            print("An error occurred while fetching the mode icon.")
            print(e)
            return None, e
        except Exception as e:
            return None, e


class Component:
    asset: Asset = Asset()

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

    def set_relative_center_x(self, dependent: "Component") -> None:
        self.set_center_x(2 * dependent.x + dependent.width)

    def set_relative_center_y(self, dependent: "Component") -> None:
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

    def _set_font(self, size: int | None) -> None:
        if size is not None:
            self.font, self.error = self.asset.font(size)
        else:
            self.font, self.error = self.asset.font(30)

    def _revert_font(self) -> None:
        self.font, self.error = self.asset.font(30)

    def _get_text_size(
        self,
        text: str = "",
        font_size: int = None,
    ) -> Tuple[int, int]:
        """
        Get the width and height of the text.

        Args:
            text (str): The text to measure.
            font_size (int): The size of the font.

        Returns:
            Tuple[int, int]: The width and height of the text.
        """
        if font_size is not None:
            self._set_font(font_size)
        draw = ImageDraw.Draw(self.img)
        bbox = draw.textbbox((0, 0), text, font=self.font, stroke_width=1)
        self._revert_font()
        text_width = bbox[2] - bbox[0]
        text_height = bbox[3] - bbox[1]
        return text_width, text_height

    def write(
        self,
        text: str = "",
        font_size: int = None,
        textbox_pos: Tuple[Tuple[int, int], Tuple[int, int]] | None = None,
        align: str = "left",
        color: Tuple[int, int, int] = (0, 0, 0),
        stroke: str = None,
    ) -> None:
        """
        Write text on an image with specified alignment.

        Args:
            text (str): The text to write.
            textbox_pos (Tuple[Tuple[int, int], Tuple[int, int]]): The position of the text box in the format ((x1, y1), (x2, y2)).
            align (str): The horizontal alignment ('left', 'center', 'right').
            color (Tuple[int, int, int]): The color of the text in RGB.
        """
        draw = ImageDraw.Draw(self.img)
        textbox_pos = textbox_pos or ((0, 0), (self.img.width, self.img.height))
        text_width, text_height = self._get_text_size(text=text, font_size=font_size)

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
        self._set_font(font_size)

        # Draw the text
        draw.text(
            xy=(x, y),
            text=text,
            font=self.font,
            fill=color,
            stroke_width=3 if stroke else 0,
            stroke_fill=stroke,
        )
        self._revert_font()

    def __str__(self) -> str:
        return f"{self.name} ({self.width}x{self.height})"


class Background:
    def __init__(
        self,
        width: Optional[int] = None,
        height: Optional[int] = None,
        bg: Image.Image = None,
        name: str = "untitled",
    ):
        self.image: Image.Image = bg.copy() or Image.new(
            "RGBA", (width or 1920, height or 1080), (0, 0, 0, 0)
        )
        if width is not None or height is not None:
            self.image = self.image.resize((width, height), Image.LANCZOS)

        self.width: int = self.image.width
        self.height: int = self.image.height
        self.name: str = name
        self.overlays: List[Component] = []

    def add_overlay(self, overlay: Component) -> None:
        self.overlays.append(overlay)

    def fabricate(self) -> Image.Image:
        final_image: Image.Image = self.image.copy()
        for overlay in self.overlays:
            top = overlay.img
            if overlay.has_transparency():
                if top.mode != "RGBA":
                    top = top.convert("RGBA")
                alpha = top.split()[3]
                final_image.paste(top, (overlay.x, overlay.y), alpha)
            else:
                final_image.paste(top, (overlay.x, overlay.y))

        return final_image


class BaseImage:
    """Inheriting this class allows you to define an async __init__.

    So you can create objects by doing something like `await BaseImage(params)`
    """

    asset: Asset = Asset()

    async def __new__(cls, *a, **kw):
        instance = super().__new__(cls)
        await instance.__init__(*a, **kw)
        return instance

    async def __init__(self, bg: Background, default_font_size: int = 30):
        self.font, self.error = self.asset.font(default_font_size)
        self.bg = bg
        self.error: Optional[Exception] = None
        self.components: List[Component] = []
        self.final: Optional[Image.Image] = None

    def set_font(self, size: int) -> None:
        self.font, self.error = self.asset.font(size)

    def revert_font(self) -> None:
        self.font, self.error = self.asset.font(30)

    def get_text_size(
        self,
        text: str = "",
        font_size: int = None,
    ) -> Tuple[int, int]:
        """
        Get the width and height of the text.

        Args:
            text (str): The text to measure.
            font_size (int): The size of the font.

        Returns:
            Tuple[int, int]: The width and height of the text.
        """
        if font_size is not None:
            self.set_font(font_size)
        draw = ImageDraw.Draw(self.bg.image)
        bbox = draw.textbbox((0, 0), text, font=self.font, stroke_width=1)
        self.revert_font()
        text_width = bbox[2] - bbox[0]
        text_height = bbox[3] - bbox[1]
        return text_width, text_height

    def write(
        self,
        text: str = "",
        font_size: int = None,
        textbox_pos: Tuple[Tuple[int, int], Tuple[int, int]] = ((0, 0), (0, 0)),
        align: str = "left",
        color: Tuple[int, int, int] = (0, 0, 0),
        stroke: str = None,
    ) -> None:
        """
        Write text on an image with specified alignment.

        Args:
            text (str): The text to write.
            textbox_pos (Tuple[Tuple[int, int], Tuple[int, int]]): The position of the text box in the format ((x1, y1), (x2, y2)).
            align (str): The horizontal alignment ('left', 'center', 'right').
            color (Tuple[int, int, int]): The color of the text in RGB.
        """
        draw = ImageDraw.Draw(self.bg.image)

        text_width, text_height = self.get_text_size(text=text, font_size=font_size)

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
        if font_size is not None:
            self.set_font(font_size)
        # Draw the text
        draw.text(
            xy=(x, y),
            text=text,
            font=self.font,
            fill=color,
            stroke_width=None if stroke is None else 3,
            stroke_fill=stroke,
        )
        self.revert_font()

    def build(self) -> None:
        for component in self.components:
            self.bg.add_overlay(component)
        self.final = self.bg.fabricate()
        self.components = []

    def _update_error(self, error):
        if error:
            self.error = error

    def resize(self, width: int) -> None:
        height = int((width / self.final.width) * self.final.height)
        self.final = self.final.resize((width, height), Image.LANCZOS)
    
    def bytes(self) -> Union[bytes, Exception]:
        output = io.BytesIO()
        if self.final is None:
            return Exception("Image not built")
        self.final.save(output, format="PNG")
        output.seek(0)
        return output.getvalue()
