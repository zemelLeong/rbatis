/// this the py_sql/html_sql syntax tree
/// * py_sql is parsed and eventually converted to an XML tree, which is then generated by the HTML Parser
/// I always believe that the tree should be used to manipulate the SQL syntax abstractly, without mixing in other framework-related content.
pub mod bind_node;
pub mod choose_node;
pub mod continue_node;
pub mod error;
pub mod foreach_node;
pub mod if_node;
pub mod otherwise_node;
pub mod set_node;
pub mod string_node;
pub mod trim_node;
pub mod when_node;
pub mod where_node;
pub mod sql_node;

use crate::codegen::syntax_tree::bind_node::BindNode;
use crate::codegen::syntax_tree::choose_node::ChooseNode;
use crate::codegen::syntax_tree::continue_node::ContinueNode;
use crate::codegen::syntax_tree::foreach_node::ForEachNode;
use crate::codegen::syntax_tree::if_node::IfNode;
use crate::codegen::syntax_tree::otherwise_node::OtherwiseNode;
use crate::codegen::syntax_tree::set_node::SetNode;
use crate::codegen::syntax_tree::string_node::StringNode;
use crate::codegen::syntax_tree::trim_node::TrimNode;
use crate::codegen::syntax_tree::when_node::WhenNode;
use crate::codegen::syntax_tree::where_node::WhereNode;
use crate::codegen::syntax_tree::sql_node::SqlNode;

/// the syntax tree enum types
#[derive(Clone, Debug)]
pub enum NodeType {
    NString(StringNode),
    NIf(IfNode),
    NTrim(TrimNode),
    NForEach(ForEachNode),
    NChoose(ChooseNode),
    NOtherwise(OtherwiseNode),
    NWhen(WhenNode),
    NBind(BindNode),
    NSet(SetNode),
    NWhere(WhereNode),
    NContinue(ContinueNode),
    NSql(SqlNode),
}

/// the node name
pub trait Name {
    fn name() -> &'static str;
}

/// node default name
pub trait DefaultName {
    fn default_name() -> &'static str;
}

/// Convert syntax tree to HTML deconstruction
pub trait AsHtml {
    fn as_html(&self) -> String;
}

impl AsHtml for StringNode {
    fn as_html(&self) -> String {
        if self.value.starts_with("`") && self.value.starts_with("`") {
            self.value.to_string()
        } else {
            format!("`{}`", self.value)
        }
    }
}

impl AsHtml for IfNode {
    fn as_html(&self) -> String {
        let mut childs = String::new();
        for x in &self.childs {
            childs.push_str(&x.as_html());
        }
        format!("<if test=\"{}\">{}</if>", self.test, childs)
    }
}

impl AsHtml for TrimNode {
    fn as_html(&self) -> String {
        let mut childs = String::new();
        for x in &self.childs {
            childs.push_str(&x.as_html());
        }
        format!(
            "<trim prefixOverrides=\"{}\" suffixOverrides=\"{}\">{}</trim>",
            self.trim, self.trim, childs
        )
    }
}

impl AsHtml for ForEachNode {
    fn as_html(&self) -> String {
        let mut childs = String::new();
        for x in &self.childs {
            childs.push_str(&x.as_html());
        }
        format!(
            "<foreach collection=\"{}\" index=\"{}\" item=\"{}\" >{}</foreach>",
            self.collection, self.index, self.item, childs
        )
    }
}

impl AsHtml for ChooseNode {
    fn as_html(&self) -> String {
        let mut childs = String::new();
        for x in &self.when_nodes {
            childs.push_str(&x.as_html());
        }
        let mut other_html = String::new();
        match &self.otherwise_node {
            None => {}
            Some(v) => {
                other_html = v.as_html();
            }
        }
        format!("<choose>{}{}</choose>", childs, other_html)
    }
}

impl AsHtml for OtherwiseNode {
    fn as_html(&self) -> String {
        let mut childs = String::new();
        for x in &self.childs {
            childs.push_str(&x.as_html());
        }
        format!("<otherwise>{}</otherwise>", childs)
    }
}

impl AsHtml for WhenNode {
    fn as_html(&self) -> String {
        let mut childs = String::new();
        for x in &self.childs {
            childs.push_str(&x.as_html());
        }
        format!("<when test=\"{}\">{}</when>", self.test, childs)
    }
}

impl AsHtml for BindNode {
    fn as_html(&self) -> String {
        format!("<bind name=\"{}\" value=\"{}\"/>", self.name, self.value)
    }
}

impl AsHtml for SetNode {
    fn as_html(&self) -> String {
        let mut childs = String::new();
        for x in &self.childs {
            childs.push_str(&x.as_html());
        }
        format!("<set>{}</set>", childs)
    }
}

impl AsHtml for WhereNode {
    fn as_html(&self) -> String {
        let mut childs = String::new();
        for x in &self.childs {
            childs.push_str(&x.as_html());
        }
        format!("<where>{}</where>", childs)
    }
}

impl AsHtml for NodeType {
    fn as_html(&self) -> String {
        match self {
            NodeType::NString(n) => n.as_html(),
            NodeType::NIf(n) => n.as_html(),
            NodeType::NTrim(n) => n.as_html(),
            NodeType::NForEach(n) => n.as_html(),
            NodeType::NChoose(n) => n.as_html(),
            NodeType::NOtherwise(n) => n.as_html(),
            NodeType::NWhen(n) => n.as_html(),
            NodeType::NBind(n) => n.as_html(),
            NodeType::NSet(n) => n.as_html(),
            NodeType::NWhere(n) => n.as_html(),
            NodeType::NContinue(n) => n.as_html(),
            NodeType::NSql(n) => n.as_html(),
        }
    }
}

impl AsHtml for Vec<NodeType> {
    fn as_html(&self) -> String {
        let mut htmls = String::new();
        for x in self {
            htmls.push_str(&x.as_html());
        }
        htmls
    }
}

pub fn to_html(args: &Vec<NodeType>, is_select: bool, fn_name: &str) -> String {
    let htmls = args.as_html();
    if is_select {
        format!(
            "<mapper><select id=\"{}\">{}</select></mapper>",
            fn_name, htmls
        )
    } else {
        format!(
            "<mapper><update id=\"{}\">{}</update></mapper>",
            fn_name, htmls
        )
    }
}
