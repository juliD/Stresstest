cargo build
cd target/debug
mpirun -np 3 ./actor_model
cd ../..