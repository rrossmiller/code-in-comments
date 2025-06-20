import asyncio

# import httpx
import httpx
from rate_limiter import RateLimiter, worker
from tqdm import tqdm

rl = RateLimiter(600)


async def f(i, bar=None, worker=None):
    async with httpx.AsyncClient() as client:
        return await client.get("http://localhost:3000", params={"i": i})


async def run(n_tasks, n_workers=10, worker_bars=True):
    task_queue = asyncio.Queue()

    for i in range(n_tasks):
        task_queue.put_nowait((f, [i]))

    loop = asyncio.get_event_loop()
    tasks: list[asyncio.Task] = []

    ttl_tracker = None
    if worker_bars:
        ttl_tracker = tqdm(position=0, total=n_tasks, desc="Tasks", unit="tasks")
    for n in range(min(n_tasks, n_workers)):
        task = loop.create_task(
            worker(n, rl, task_queue, ttl_tracker=ttl_tracker, worker_bars=worker_bars)
        )
        tasks.append(task)

    # wait for workers to finish jobs
    await task_queue.join()

    if worker_bars:
        ttl_tracker.close()

    # collate task results
    results = []
    for t in tasks:
        r: list = t.result()
        if r[0] is not None:
            r[0].close()
        for x in r[1]:
            results.append(x.json()['id'])


    print(results)


if __name__ == "__main__":
    r = httpx.delete("http://localhost:3000").json()
    assert len(r) == 0
    print("cleared:", r)
    asyncio.run(run(1200))

    """
    2 mins at
    600 requests per minute
    is  1200 requests
    """
