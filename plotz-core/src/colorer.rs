use crate::bucket::Bucket;
use plotz_color::*;
use thiserror::Error;

trait Colorer {
    type Bucket;
    type Color;
    type Error;
    /// Given a bucket, apply a color or return an error.
    fn color(&self, bucket: Self::Bucket) -> Result<Self::Color, Self::Error>;
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ColorerError {
    #[error("could not color")]
    ColorerError,
}

pub struct DefaultColorer {
    statements: Vec<(Bucket, ColorRGB)>,
}

impl DefaultColorer {
    pub fn new(statements: impl IntoIterator<Item = (Bucket, ColorRGB)>) -> DefaultColorer {
        DefaultColorer {
            statements: statements.into_iter().collect(),
        }
    }
}

impl Colorer for DefaultColorer {
    type Bucket = Bucket;
    type Color = ColorRGB;
    type Error = ColorerError;
    fn color(&self, b: Self::Bucket) -> Result<Self::Color, Self::Error> {
        self.statements
            .iter()
            .find_map(|(bucket, color)| if *bucket == b { Some(*color) } else { None })
            .ok_or(ColorerError::ColorerError)
    }
}

#[cfg(test)]
mod test_super {
    use super::*;
    use crate::bucket::Path;
    use crate::colorer_builder::DefaultColorerBuilder;

    #[test]
    fn test_default_colorer_builder() {
        let c = DefaultColorerBuilder::default();
        assert_eq!(c.color(Bucket::Path(Path::Boundary)), Ok(LIGHTGRAY));
    }

    #[test]
    fn test_colorer_new() {
        let c = DefaultColorer::new(vec![]);
        assert_eq!(
            c.color(Bucket::Path(Path::Boundary)),
            Err(ColorerError::ColorerError)
        );
    }
}
