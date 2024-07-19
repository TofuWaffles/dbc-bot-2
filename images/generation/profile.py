import base64
from typing import Union

from PIL.Image import Resampling
from pydantic import BaseModel

from .model import Background, BaseImage, Component


class User(BaseModel):
    discord_id: str
    discord_name: str
    icon: int
    player_name: str
    player_tag: str
    trophies: int
    brawler_count: int
    tournament_id: str


class RequestProfile(BaseModel):
    player: User

    async def respond(self) -> Union[str, Exception]:
        image = await Profile(self.player)
        if image.error:
            return image.error
        image.preset()
        image.build()
        bytes = image.bytes()
        if bytes is Exception:
            return Exception("Error while converting image to bytes")
        encode = base64.b64encode(bytes).decode("utf-8")
        return encode


class Profile(BaseImage):
    async def __init__(self, player: User):
        bg, self.error = self.asset.get_image("Profile.png")
        await super().__init__(
            bg=Background(width=None, height=None, bg=bg, name="Profile")
        )
        self.player: User = player
        self.layer, error = self.asset.get_image("Layer.png")
        self._update_error(error)
        self.icon, error = await self.asset.icon(self.player.icon)
        self._update_error(error)
        self.brawler, error = self.asset.get_image("28000000.png")
        self._update_error(error)
        layer = Component(img=self.layer, pos=(0, 0), name="Layer")
        self.bg.image.paste(im=layer.img, box=(layer.x, layer.y), mask=layer.img)

    def preset(self) -> None:
        icon = Component(
            img=self.icon.resize((173, 173), resample=Resampling.NEAREST),
            pos=(25, 22),
            name="Icon",
        )
        bicon = Component(
            img=self.brawler.resize((87, 87), resample=Resampling.NEAREST),
            pos=(25, 214),
            name="Brawler",
        )

        self.write(
            text=self.player.discord_name,
            textbox_pos=((198, 22), (646, 68)),
            align="center",
            color=(255, 255, 255),
            stroke="black",
        )
        self.write(
            text=self.player.discord_id,
            textbox_pos=((198, 73), (478, 98)),
            font_size=20,
            align="center",
            color=(255, 255, 255),
            stroke="black",
        )
        self.write(
            text=self.player.player_name,
            textbox_pos=((198, 120), (446, 165)),
            align="center",
            color=(255, 255, 255),
            stroke="black",
        )
        self.write(
            text=self.player.player_tag,
            textbox_pos=((200, 170), (360, 195)),
            font_size=20,
            align="left",
            color=(255, 255, 255),
            stroke="black",
        )
        self.write(
            text=str(self.player.brawler_count),
            textbox_pos=((113, 215), (233, 260)),
            align="center",
            color=(255, 255, 255),
            stroke="black",
        )
        self.write(
            text=str(self.player.trophies),
            textbox_pos=((472, 120), (660, 165)),
            align="right",
            color=(255, 255, 255),
            stroke="black",
        )
        self.write(
            text=self.player.tournament_id,
            textbox_pos=((472, 215), (660, 260)),
            align="right",
            color=(255, 255, 255),
            stroke="black",
        )
        self.components.extend([icon, bicon])
