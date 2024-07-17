from fastapi import FastAPI, Response, HTTPException
from generation.match import RequestMatch
from generation.profile import RequestProfile
app = FastAPI()

@app.get("/image/match")
async def match(image: RequestMatch):
    data = await image.respond()
    if isinstance(data, Exception):
        raise HTTPException(status_code=400, detail=str(data))
    return Response(content=data, media_type="image/png")
    
@app.get("/image/profile")
async def profle(image: RequestProfile):
    data = await image.respond()
    return Response(content=data, media_type="image/png")
    