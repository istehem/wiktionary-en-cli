use colored::Colorize;
use colored::ColoredString;
use textwrap::fill;

pub trait Join {
    fn join(&self, list : Vec<Self>) -> Self where Self: Sized;
    fn joinwrap(&self, list : Vec<Self>, width : usize) -> Self where Self: Sized;
}

impl Join for ColoredString {
    fn join(&self, list : Vec<ColoredString>) -> ColoredString {
        let mut res : ColoredString = "".normal();
        let len : usize = list.len();
        for (i, string) in list.iter().enumerate() {
            res = format!("{}{}", res, string).normal();
            if i < len - 1 {
                res = format!("{}{}", res, self).normal();
            }
        }
        return res.clone();
    }

    fn joinwrap(&self, list : Vec<ColoredString>, width : usize) -> ColoredString {
        let text = self.join(list);
        return fill(&text, width).normal();
    }
}

