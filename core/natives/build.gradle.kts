plugins {
    id("fr.stardustenterprises.rust.wrapper") version "3.2.4"
}

rust {
    release.set(true)

    targets += defaultTarget()
}