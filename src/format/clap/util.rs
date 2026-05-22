use super::{BuildClapInfo, ClapInfo, ClapPlugin};

pub fn with_clap_info<P, F>(f: F)
where
    P: ClapPlugin,
    F: FnOnce(ClapInfo),
{
    struct BuildClapInfoFn<F>(F);

    impl<F> BuildClapInfo for BuildClapInfoFn<F>
    where
        F: FnOnce(ClapInfo),
    {
        fn info(self, info: ClapInfo) {
            self.0(info)
        }
    }

    P::clap_info(BuildClapInfoFn(f))
}
