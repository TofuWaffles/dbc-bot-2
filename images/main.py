from fastapi import FastAPI, HTTPException
from fastapi.responses import PlainTextResponse
from generation.battle_log import RequestBattleLog
from generation.match import RequestMatch
from generation.profile import RequestProfile
from generation.result import RequestResult

app = FastAPI(
    title="Image generation",
    description="This is where you can generate images",
    version="1.0.0",
    license_info={
        "name": "Apache 2.0",
        "url": "http://www.apache.org/licenses/LICENSE-2.0.html",
    },
)


@app.get("/image/match", response_class=PlainTextResponse)
async def match(image: RequestMatch):
    data = await image.respond()
    if isinstance(data, Exception):
        raise HTTPException(status_code=500, detail=str(data))
    return data


@app.get("/image/profile", response_class=PlainTextResponse)
async def profle(image: RequestProfile):
    data = await image.respond()
    if isinstance(data, Exception):
        raise HTTPException(status_code=500, detail=str(data))
    return data


@app.get("/image/result", response_class=PlainTextResponse)
async def result(image: RequestResult):
    data = await image.respond()
    if isinstance(data, Exception):
        raise HTTPException(status_code=500, detail=str(data))
    return data


@app.get("/image/battle_log", response_class=PlainTextResponse)
async def battle_log(image: RequestBattleLog):
    data = await image.respond()
    if isinstance(data, Exception):
        raise HTTPException(status_code=500, detail=str(data))
    return data


# if __name__ == "__main__":
#     DEFAULT = "127.0.0.1"
#     DOCKER = "0.0.0.0"

#     def running_in_docker() -> bool:
#         return os.path.exists("/.dockerenv")

#     runInDocker = running_in_docker()
#     PORT: int = int(os.getenv("PORT"))
#     if not runInDocker:
#         from dotenv import load_dotenv

#         load_dotenv()
#     else:
#         uvicorn.run(app, host=DOCKER if runInDocker else DEFAULT, port=PORT)
