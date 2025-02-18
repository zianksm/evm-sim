from nox import session

@session()
def tests(session):
    session.env["RUST_BACKTRACE"] = "1"
    session.run("maturin","develop")
    session.env["MATURIN_PEP517_ARGS"] = "--profile=dev"
    session.install(".[dev]")
    session.install("web3")
    session.run("pytest")