import base64
from typing import Union

from PIL.Image import Resampling
from pydantic import BaseModel

from .model import Background, BaseImage, Component, Player


class RequestMatch(BaseModel):
    player1: Player
    player2: Player

    async def respond(self) -> Union[str, Exception]:
        image = await Match(self.player1, self.player2)
        if image.error:
            return image.error
        image.preset()
        image.build()
        encode = base64.b64encode(image.bytes()).decode("utf-8")
        return encode


class Match(BaseImage):
    async def __init__(self, player1: Player, player2: Player):
        bg, self.error = self.asset.get_image("Player_Player_clean.png")
        await super().__init__(bg=Background(None, None, bg, "Match"))
        self.player1: Player = player1
        self.player2: Player = player2
        self.pi1, self.error = await self.asset.icon(self.player1.icon)
        self.pi2, self.error = await self.asset.icon(self.player2.icon)

    def preset(self) -> None:
        ICON_SIZE = (275, 275)
        icon1 = Component(
            img=self.pi1.resize(size=ICON_SIZE, resample=Resampling.NEAREST),
            pos=(175, 175),
            name="Icon1",
        )
        icon2 = Component(
            img=self.pi2.resize(size=ICON_SIZE, resample=Resampling.NEAREST),
            pos=(830, 175),
            name="Icon2",
        )
        self.write(
            text=self.player1.discord_name,
            textbox_pos=((125, 460), (500, 530)),
            align="center",
            color="white",
        )
        self.write(
            text=self.player2.discord_name,
            textbox_pos=((780, 460), (1155, 530)),
            align="center",
            color="white",
        )
        self.components.extend([icon1, icon2])
