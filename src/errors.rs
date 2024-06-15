use iris::IrisError;
use snafu::Snafu;

use crate::constants::IRIS_SECRET_ENV_VAR;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display(
        "Unable to read room identifier, consider setting the {IRIS_SECRET_ENV_VAR} env variable"
    ))]
    MissingRoomIdentifier { source: dialoguer::Error },
    #[snafu(display(
        "unable to read passphrase, consider setting the {IRIS_SECRET_ENV_VAR} env variable"
    ))]
    MissingPassphrase { source: dialoguer::Error },
    #[snafu(transparent)]
    Iris { source: IrisError },
}
