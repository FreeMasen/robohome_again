psql robohome_again -f ../../migrations/initial.down.sql && psql robohome_again -f ../../migrations/initial.sql && psql robohome_again -f ../../migrations/initial.seed.sql && cargo run