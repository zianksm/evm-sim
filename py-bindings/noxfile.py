from nox import session

@session()
def tests(session):
    session.install(".",)
    session.install("pytest", "pytest-cov")
    session.run("pytest")