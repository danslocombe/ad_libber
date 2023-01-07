pub struct QueueParams<'a>
{
    pub filename: &'a str,
    pub section: &'a str,
    pub oneshot: bool,
}

impl<'a> QueueParams<'a> {
    pub fn parse(input : &'a str) -> Option<Self> {
        let mut splits = input.split("|").map(|x| x);
        let filename = splits.next()?;
        let section = splits.next()?;
        let mut oneshot = false;

        while let Some(arg) = splits.next() {
            if (unicase::eq_ascii(arg, "oneshot")) {
                oneshot = true;
            }
        }

        Some(Self {
            filename,
            section,
            oneshot,
        })
    }
}