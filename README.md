## Actor Model

### Vorrausetzungen
- Docker
- OpenMPI (`brew install open-mpi`)


### Lokale Entwicklung (Ein Thread)
- Ihr könnt ganz normal lokal im rust/ Ordner entwickeln
- `cargo run` führt dann einen Prozess aus

### Lokale Entwicklung (Mehrere Threads auf einen Rechner)
- Nachdem ihr das Projekt gebuildet habt
- führt ihr folgenden Befehl aus: `mpirun -np 5 ./rust/target/debug/actor_model` 

Hier Ein Beispiel:
```
Toms-MBP:Rust tomesders$ pwd
/Users/tomesders/Desktop/Rust
Toms-MBP:Rust tomesders$ ls -la
total 16
drwxr-xr-x   7 tomesders  staff  224 27 Jan 17:56 .
drwx------+ 13 tomesders  staff  416 27 Jan 17:55 ..
drwxr-xr-x  12 tomesders  staff  384 27 Jan 17:56 .git
-rw-r--r--   1 tomesders  staff    9 27 Jan 17:58 README.md
drwxr-xr-x   8 tomesders  staff  256 27 Jan 17:53 docker
-rwxr-xr-x   1 tomesders  staff  627 27 Jan 17:52 run.sh
drwxr-xr-x@  8 tomesders  staff  256 27 Jan 17:51 rust
Toms-MBP:Rust tomesders$ mpirun -np 5 ./rust/target/debug/actor_model
Hello from processor : Toms-MBP.fritz.box, process 0 of 5! Starting Request..
Hello from processor : Toms-MBP.fritz.box, process 1 of 5! Starting Request..
Hello from processor : Toms-MBP.fritz.box, process 4 of 5! Starting Request..
Hello from processor : Toms-MBP.fritz.box, process 2 of 5! Starting Request..
Hello from processor : Toms-MBP.fritz.box, process 3 of 5! Starting Request..
Response from Processor(Toms-MBP.fritz.box), Process(2): HTTP Response 200 OK
Response from Processor(Toms-MBP.fritz.box), Process(4): HTTP Response 200 OK
Response from Processor(Toms-MBP.fritz.box), Process(0): HTTP Response 200 OK
Response from Processor(Toms-MBP.fritz.box), Process(3): HTTP Response 200 OK
Response from Processor(Toms-MBP.fritz.box), Process(1): HTTP Response 200 OK
```

### Testing auf mehreren Rechnern
- Ich habe eine `run.sh` vorbereitet welche alle Schritte selbständig ausführen sollte
- der Befehl lautet also: `sh run.sh`
- Dieser baut auotmatisch einen Docker-Container, auf welchen des Rust-Projekt kompiliert und verteilt wird. Auch die Hosts werden automatisch erkannt.
- Ihr könnt die Anzahl der Rechner in der `run.sh` einstellen, ändert dafür in der Zeile `docker-compose up --scale node=4 -d`  den Parameter node={ANZAHL)