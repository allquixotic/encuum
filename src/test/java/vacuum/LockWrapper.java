package vacuum;

import java.util.concurrent.locks.Lock;

public class LockWrapper implements AutoCloseable
{
    private final Lock _lock;
    public LockWrapper(Lock l) {
       this._lock = l;
    }

    public void lock() {
        this._lock.lock();
    }

    public void close() {
        this._lock.unlock();
    }
}