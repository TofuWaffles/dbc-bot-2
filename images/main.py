from fastapi import FastAPI, HTTPException
from fastapi.responses import PlainTextResponse
from generation.match import RequestMatch
from generation.result import RequestResult
from generation.profile import RequestProfile
import os
import uvicorn

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


if __name__ == "__main__":

    def running_in_docker() -> bool:
        return os.path.exists("/.dockerenv")

    if not running_in_docker():
        from dotenv import load_dotenv

        load_dotenv()
    PORT: int = int(os.getenv("PORT"))
    uvicorn.run(app, host="0.0.0.0", port=PORT)
