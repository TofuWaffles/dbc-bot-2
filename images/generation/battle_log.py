import base64
from typing import List, Union

from PIL import Image
from PIL.Image import Resampling
from pydantic import BaseModel

from .model import Background, BaseImage, Component, ExtendedPlayer, Mode


class BattleLog(BaseModel):
    winner: ExtendedPlayer
    eliminated: ExtendedPlayer
    battle_time: str
    duration: int
    mode: str
    map: str
    type: str
    result: str | None  # winner discord id or null if draw


class TempPlayer:
    discord_id: str
    discord_name: str
    icon: Image.Image

    def __init__(self, discord_id: str, discord_name: str, icon: Image.Image):
        self.discord_id = discord_id
        self.discord_name = discord_name
        self.icon = icon


class RequestBattleLog(BaseModel):

    battle_logs: List[BattleLog]

    async def respond(self) -> Union[str, Exception]:
        print(self)
        image = await BattleLog(self.battle_logs)
        if image.error:
            print(image.error)
            return image.error
        image.preset()
        image.build()
        image.resize(width=500)
        bytes = image.bytes()
        if bytes is Exception:
            return Exception("Error while converting image to bytes")
        encode = base64.b64encode(bytes).decode("utf-8")
        return encode


class BattleLog(BaseImage):
    async def __init__(self, battle_logs: List[BattleLog]):
        bg, self.error = self.asset.get_image("battle_log_bg.png")
        self.battle_logs = battle_logs
        vs, self.error = self.asset.get_image("Vs_sign.png")
        self.vs = vs
        n = len(battle_logs)
        self.gap = 10
        await super().__init__(
            bg=Background(
                width=1280, height=720 * n + self.gap * (n - 1), bg=bg, name="BattleLog"
            )
        )
        self.mode_icon, self.error = await self.asset.get_mode_icon(
            mode=battle_logs[0].mode
        )
        self.winner: TempPlayer = TempPlayer(
            discord_id=battle_logs[0].winner.discord_id,
            discord_name=battle_logs[0].winner.discord_name,
            icon=(await self.asset.icon(battle_logs[0].winner.icon))[
                0
            ],  # idk why this is a tuple lol
        )
        self.eliminated: TempPlayer = TempPlayer(
            discord_id=battle_logs[0].eliminated.discord_id,
            discord_name=battle_logs[0].eliminated.discord_name,
            icon=(await self.asset.icon(battle_logs[0].eliminated.icon))[0],
        )

    def preset(self) -> None:
        for index, log in enumerate(self.battle_logs):
            result = "Draw" if log.result is None else f"{self.winner.discord_name} won this round"
            base_bg = Component(
                img=Image.new(mode="RGB", size=(1280, 720), color="#8274A0"),
                pos=(0, 720 * index + self.gap * (index + 1)),
                name=f"BaseBG{index}",
            )

            base_bg.write(
                text=log.battle_time,
                font_size=30,
                textbox_pos=((0, 0), (1280, 48)),
                align="right",
                color=(255, 255, 255),
            )

            layer1 = Component(
                img=Image.new(mode="RGB", size=(1280, 96), color="#786C98"),
                pos=(0, base_bg.y + 48),
            )
            layer1.write(
                text=f"{Mode.name(log.mode)}\n{log.map}",
                textbox_pos=((100, 0), (1280, 96)),
                font_size=30,
                align="left",
                color="white",
            )
            layer1.write(
                text=result,
                font_size=40,
                align="center",
                color="yellow",
            )
            mode_icon = Component(
                img=self.mode_icon.resize(size=(90, 90), resample=Resampling.NEAREST),
                pos=(0, base_bg.y + 48),
                name="ModeIcon",
            )

            padding = 50
            icon_size = (200, 200)
            winner_icon = Component(
                name=self.winner.discord_name,
                img=self.winner.icon.resize(size=icon_size, resample=Resampling.NEAREST),
                pos=(padding, base_bg.y + 200),
            )
            eliminated_icon = Component(
                name=self.eliminated.discord_name,
                img=self.eliminated.icon.resize(size=icon_size, resample=Resampling.NEAREST).convert(mode="L"),
                pos=(1280 - padding - icon_size[0], base_bg.y + 200),
            )
            base_bg.write(
                text=winner_icon.name,
                font_size=30,
                textbox_pos=(
                    (padding, icon_size[1] + padding),
                    (padding + icon_size[0], 600),
                ),
                align="center",
                color="white",
            )
            base_bg.write(
                text=eliminated_icon.name,
                font_size=30,
                textbox_pos=(
                    (1280 - padding - icon_size[0], icon_size[1] + padding),
                    (1280 - padding, 600),
                ),
                align="center",
                color="white",
            )
            vs = Component(
                img=self.vs,
                pos=(0, 0),
                name="Vs",
            )
            vs.set_x(base_bg.x + (base_bg.width - vs.img.width) // 2)
            vs.set_y(base_bg.y + (base_bg.height - vs.img.height) // 2)
            self.components.extend(
                [base_bg, layer1, mode_icon, winner_icon, eliminated_icon, vs]
            )
